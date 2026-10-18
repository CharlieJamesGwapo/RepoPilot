use crate::{
    cli::AiOptions,
    config::{Config, ProviderConfig, is_loopback},
};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
const LIMIT: usize = 1_048_576;
#[derive(Serialize)]
pub struct Capabilities {
    pub text_generation: bool,
    pub model_discovery: bool,
    pub streaming: bool,
    pub structured_output: bool,
    pub tool_calling: bool,
    pub cancellation: &'static str,
    pub context_limit: Option<usize>,
    pub authentication: &'static str,
    pub execution: &'static str,
}
pub trait Provider {
    fn capabilities(&self) -> Capabilities;
    fn generate(&self, prompt: &str) -> Result<String>;
    fn models(&self) -> Result<Vec<String>>;
}
pub struct Api<'a> {
    pub config: &'a ProviderConfig,
}
pub struct Process<'a> {
    pub config: &'a ProviderConfig,
}
fn caps(models: bool, auth: &'static str, execution: &'static str) -> Capabilities {
    Capabilities {
        text_generation: true,
        model_discovery: models,
        streaming: false,
        structured_output: false,
        tool_calling: false,
        cancellation: "Ctrl-C terminates RepoPilot; requests have a timeout",
        context_limit: None,
        authentication: auth,
        execution,
    }
}
pub fn capabilities(p: &ProviderConfig) -> Capabilities {
    if p.kind == "process" {
        Process { config: p }.capabilities()
    } else {
        Api { config: p }.capabilities()
    }
}
pub fn selected<'a>(
    c: &'a Config,
    o: &AiOptions,
    explicit: bool,
) -> Result<(&'a str, &'a ProviderConfig)> {
    if !explicit {
        bail!(
            "Provider execution requires an explicitly trusted --config PATH; offline scan, explain and doctor remain available"
        )
    }
    let name = o
        .provider
        .as_ref()
        .or(c.active.as_ref())
        .context("No provider selected; configure one, or use scan/explain/doctor locally")?;
    let (name, p) = c
        .providers
        .get_key_value(name)
        .context("Selected provider is not configured")?;
    if o.local
        && (p.kind == "process"
            || !p
                .endpoint
                .as_ref()
                .and_then(|s| url::Url::parse(s).ok())
                .is_some_and(|u| is_loopback(&u)))
    {
        bail!(
            "local mode rejects remote endpoints and process adapters; use a literal loopback API or deterministic scan/explain"
        )
    }
    if p.kind == "process" {
        if !o.allow_exec {
            bail!(
                "Process execution requires consent: review executable/arguments in config then use --allow-exec"
            )
        }
    } else if !is_loopback(&url::Url::parse(p.endpoint.as_deref().unwrap_or(""))?)
        && !o.allow_external
    {
        bail!(
            "External transfer requires consent: review the configured endpoint then pass --allow-external; only your typed prompt is sent"
        )
    }
    if p.kind != "process" {
        eprintln!(
            "RepoPilot: authorized API request; provider {name}; endpoint {}. Payload: typed prompt or explicit probe/model-list request only; no project files or history.",
            p.endpoint.as_deref().unwrap_or("unknown")
        );
    }
    Ok((name, p))
}
pub fn adapter(p: &ProviderConfig) -> Box<dyn Provider + '_> {
    if p.kind == "process" {
        Box::new(Process { config: p })
    } else {
        Box::new(Api { config: p })
    }
}
pub fn clean(text: &str, p: &ProviderConfig) -> String {
    let mut out = text.to_owned();
    if let Some(key) = p
        .key_env
        .as_ref()
        .and_then(|n| std::env::var(n).ok())
        .filter(|s| !s.is_empty())
    {
        out = out.replace(&key, "[REDACTED]");
    }
    out.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .take(LIMIT)
        .collect()
}
impl Api<'_> {
    fn request(&self, suffix: &str, body: Option<Value>) -> Result<Value> {
        let p = self.config;
        let endpoint = format!(
            "{}/{}",
            p.endpoint
                .as_deref()
                .context("Missing endpoint")?
                .trim_end_matches('/'),
            suffix
        );
        let agent = ureq::Agent::new_with_config(
            ureq::Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(p.timeout)))
                .proxy(None)
                .max_redirects(0)
                .http_status_as_error(false)
                .build(),
        );
        let token = match &p.key_env {
            Some(name) => Some(std::env::var(name).ok().filter(|s| !s.is_empty()).context(
                "Configured credential environment variable is missing or empty; set it in your shell",
            )?),
            None => None,
        };
        let response = if let Some(body) = body {
            let mut r = agent.post(&endpoint);
            if let Some(t) = &token {
                r = r.header("Authorization", format!("Bearer {t}"));
            }
            r.send_json(body)
        } else {
            let mut r = agent.get(&endpoint);
            if let Some(t) = &token {
                r = r.header("Authorization", format!("Bearer {t}"));
            }
            r.call()
        };
        let mut response=response.map_err(|_|anyhow::anyhow!("Provider connection failed or timed out; check endpoint, service and timeout (response details suppressed)"))?;
        if !response.status().is_success() {
            bail!(
                "Provider returned HTTP {}; check authentication, model and endpoint. Response body suppressed",
                response.status().as_u16()
            )
        }
        response.body_mut().with_config().limit(LIMIT as u64).read_json::<Value>().map_err(|_|anyhow::anyhow!("Provider response is oversized or invalid JSON; expected a supported API response"))
    }
}
impl Provider for Api<'_> {
    fn capabilities(&self) -> Capabilities {
        let p = self.config;
        caps(
            true,
            if p.key_env.is_some() {
                "bearer token via configured environment variable"
            } else {
                "none"
            },
            if p.endpoint
                .as_ref()
                .and_then(|s| url::Url::parse(s).ok())
                .is_some_and(|u| is_loopback(&u))
            {
                "loopback API"
            } else {
                "remote API"
            },
        )
    }
    fn generate(&self, prompt: &str) -> Result<String> {
        if prompt.len() > 65536 {
            bail!("Prompt exceeds 64 KiB limit")
        }
        let v=self.request("chat/completions",Some(json!({"model":self.config.model,"messages":[{"role":"user","content":prompt}],"stream":false})))?;
        let s = v
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .context("Unsupported completion response: expected choices[0].message.content text")?;
        Ok(clean(s, self.config))
    }
    fn models(&self) -> Result<Vec<String>> {
        let v = self.request("models", None)?;
        let items = v
            .get("data")
            .and_then(Value::as_array)
            .context("Unsupported models response: expected data array")?;
        let mut out: Vec<_> = items
            .iter()
            .filter_map(|x| x.get("id").and_then(Value::as_str))
            .map(|s| clean(s, self.config))
            .collect();
        out.sort();
        out.dedup();
        Ok(out)
    }
}
impl Provider for Process<'_> {
    fn capabilities(&self) -> Capabilities {
        caps(
            false,
            "installed tool manages its own authentication",
            "external process; network behavior unknown",
        )
    }
    fn models(&self) -> Result<Vec<String>> {
        bail!(
            "Model discovery is unsupported by the generic process protocol; configure the installed tool directly"
        )
    }
    fn generate(&self, prompt: &str) -> Result<String> {
        if prompt.len() > 65536 {
            bail!("Prompt exceeds 64 KiB limit")
        }
        let p = self.config;
        let mut child = Command::new(p.executable.as_deref().context("Missing executable")?)
            .args(&p.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| {
                anyhow::anyhow!(
                    "Cannot start configured executable; check installation and permissions"
                )
            })?;
        let mut stdin = child.stdin.take().context("Cannot open process stdin")?;
        let stdout = child.stdout.take().context("Cannot open process stdout")?;
        let prompt = prompt.to_owned();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let r = stdin.write_all(prompt.as_bytes());
            drop(stdin);
            let _ = tx.send(r.is_ok());
        });
        let (tx, output) = mpsc::channel();
        thread::spawn(move || {
            let mut bytes = Vec::new();
            let r = stdout.take((LIMIT + 1) as u64).read_to_end(&mut bytes);
            let _ = tx.send((r.is_ok(), bytes));
        });
        let start = Instant::now();
        let mut result = None;
        let mut status = None;
        let mut delivered = None;
        loop {
            if result.is_none() {
                result = output.try_recv().ok();
            }
            if delivered.is_none() {
                delivered = rx.try_recv().ok();
            }
            if status.is_none() {
                match child.try_wait() {
                    Ok(s) => status = s,
                    Err(_) => {
                        let _ = child.kill();
                        let _ = child.wait();
                        bail!("Cannot monitor configured process")
                    }
                }
            }
            if result.as_ref().is_some_and(|(_, b)| b.len() > LIMIT) {
                let _ = child.kill();
                let _ = child.wait();
                bail!("Process output exceeded 1 MiB limit")
            }
            if status.is_some() && result.is_some() && delivered.is_some() {
                break;
            }
            if start.elapsed() >= Duration::from_secs(p.timeout) {
                let _ = child.kill();
                let _ = child.wait();
                bail!("Process timed out; direct child terminated. Descendants are not managed")
            }
            thread::sleep(Duration::from_millis(10));
        }
        if !status.context("Missing process status")?.success() {
            bail!("Configured process failed; stderr suppressed to protect credentials")
        }
        if delivered != Some(true) {
            bail!("Could not deliver prompt to process stdin")
        }
        let (ok, bytes) = result.context("No process output")?;
        if !ok {
            bail!("Could not read process output")
        }
        let text = String::from_utf8(bytes).context("Process output must be UTF-8 text")?;
        Ok(clean(&text, p))
    }
}
pub fn list(c: &Config) -> Value {
    let configured: Vec<_> = c.providers.iter().map(|(name, p)| {
        let credential_present = p.key_env.as_ref().is_some_and(|n| {
            std::env::var(n).is_ok_and(|s| !s.is_empty())
        });
        let executable_present = p.executable.as_deref().is_some_and(|e| {
            std::path::Path::new(e).is_file() || crate::project::executable(e).is_some()
        });
        json!({
            "name": name, "kind": p.kind, "endpoint": p.endpoint, "model": p.model,
            "credential_configured": credential_present,
            "executable_available": executable_present,
            "capabilities": capabilities(p),
            "capabilities_scope": "adapter protocol support; endpoint capabilities are unverified until exercised",
            "connectivity": "not tested", "task_execution": "not tested"
        })
    }).collect();
    json!({
        "active": c.active, "configured": configured,
        "available_adapters": [
            "api: OpenAI-compatible /chat/completions + /models",
            "local: same protocol, literal loopback only",
            "process: explicitly configured stdin/stdout executable"
        ]
    })
}
