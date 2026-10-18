use crate::{
    cli::AiOptions,
    config::{self, Config},
    output, project, providers,
};
use anyhow::Result;
use std::{
    io::{self, BufRead, Read, Write},
    path::Path,
};
fn line(prompt: &str) -> Result<Option<String>> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut s = String::new();
    let n = io::stdin().lock().take(65538).read_line(&mut s)?;
    if n == 0 {
        return Ok(None);
    }
    if s.len() > 65536 {
        anyhow::bail!("Input exceeds 64 KiB limit")
    };
    Ok(Some(s.trim().to_owned()))
}
fn yes(prompt: &str) -> Result<bool> {
    Ok(line(prompt)?.is_some_and(|x| x == "yes"))
}
pub fn run(
    mut c: Config,
    path: &Path,
    explicit: bool,
    local: bool,
    user_config: bool,
    mut max_output_tokens: Option<u32>,
) -> Result<()> {
    let trusted = explicit || user_config;
    let mut last_usage: Option<providers::Usage> = None;
    let mut request_count = 0u64;
    let mut local = local || user_config;
    output::emit(
        "RepoPilot chat. /help for commands. No provider required; no files are sent.\nEach prompt is independent; no conversation history is transferred.",
    )?;
    if user_config {
        output::emit(
            "Using saved user configuration in local-only mode. Remote endpoints and process adapters are blocked for this session.",
        )?;
    }
    while let Some(input) = line("repopilot> ")? {
        let (cmd, arg) = input.split_once(' ').unwrap_or((&input, ""));
        let result: Result<()> = (|| match cmd {
            "/exit" | "/quit" => Ok(()),
            "/help" => output::emit(
                "/providers /connect /use <name> /models /status /budget [1..32768|off] /usage /local /disconnect /scan [path] /doctor [path] /explain [path] /exit\n/connect guides setup; /use only selects a provider, without testing or authenticating. /local rejects remote and process providers. Prompts require per-request consent for remote or process execution.",
            ),
            "/budget" => {
                if arg.is_empty() {
                    return output::json(
                        &serde_json::json!({"max_output_tokens":max_output_tokens,"enforcement":"requested from compatible HTTP provider; not a billing guarantee"}),
                    );
                }
                if arg == "off" {
                    max_output_tokens = None;
                } else {
                    let limit = arg
                        .parse::<u32>()
                        .ok()
                        .filter(|v| (1..=32768).contains(v))
                        .ok_or_else(|| anyhow::anyhow!("Use /budget 1..32768 or /budget off"))?;
                    max_output_tokens = Some(limit);
                }
                output::emit("Session response budget updated; saved configuration unchanged.")
            }
            "/usage" => output::json(
                &serde_json::json!({"successful_requests":request_count,"last_usage":last_usage,"source":"provider-reported only; null means unavailable; no savings estimate"}),
            ),
            "/providers" => output::json(&providers::list(&c)),
            "/status" => output::json(
                &serde_json::json!({"local_only":local,"active":c.active,"explicit_config":explicit,"config_source":if user_config {"user"} else {"explicit"},"config_path":path,"trusted_config":trusted,"connectivity":"not tested by status","context":"typed prompt only; no files; no history","providers":providers::list(&c)}),
            ),
            "/local" => {
                local = true;
                output::emit(
                    "Local-only mode enabled. Remote endpoints and process adapters are blocked; /scan, /doctor, /explain remain available.",
                )
            }
            "/disconnect" => {
                c.active = None;
                output::emit(
                    "Session selection cleared. Credentials remain in your environment; no credentials were removed.",
                )
            }
            "/use" => {
                if c.providers.contains_key(arg) {
                    c.active = Some(arg.into());
                    output::emit(
                        "Provider selected for this session; connectivity and authentication not yet tested. Local-only policy is unchanged.",
                    )
                } else {
                    anyhow::bail!("Unknown provider; /providers lists configured names")
                }
            }
            "/connect" => connect(&mut c, path, trusted, local, user_config, max_output_tokens),
            "/scan" | "/doctor" | "/explain" => {
                let r = project::scan(
                    Path::new(if arg.is_empty() { "." } else { arg }),
                    &c,
                    cmd == "/doctor",
                )?;
                output::render(&r, crate::cli::Format::Terminal)
            }
            "/models" => {
                let mut o = AiOptions {
                    local,
                    ..Default::default()
                };
                consent(&c, &mut o, "GET model identifiers (no prompt or files)")?;
                let (_, p) = providers::selected(&c, &o, trusted)?;
                output::json(&providers::adapter(p).models()?)
            }
            "" => Ok(()),
            _ if cmd.starts_with('/') => anyhow::bail!("Unknown slash command; use /help"),
            _ => {
                if c.active.is_none() {
                    return output::emit(
                        "No provider selected. Use /scan or /explain for deterministic help, or /connect and /use <name> for AI.",
                    );
                }
                let mut o = AiOptions {
                    local,
                    ..Default::default()
                };
                consent(
                    &c,
                    &mut o,
                    &format!(
                        "the typed prompt only ({} bytes); no files or history",
                        input.len()
                    ),
                )?;
                let (_, p) = providers::selected(&c, &o, trusted)?;
                let generation =
                    providers::adapter(p).generate_with_limit(&input, max_output_tokens)?;
                last_usage = generation.usage;
                request_count = request_count.saturating_add(1);
                output::emit(&generation.text)?;
                if generation.finish_reason.as_deref() == Some("length") {
                    output::emit(
                        "[Response stopped at the provider's token limit; use /budget to adjust.]",
                    )?;
                }
                Ok(())
            }
        })();
        if cmd == "/exit" || cmd == "/quit" {
            break;
        }
        if let Err(e) = result {
            eprintln!("RepoPilot: {e}");
        }
    }
    Ok(())
}
fn consent(c: &Config, o: &mut AiOptions, context: &str) -> Result<()> {
    let p = c
        .active
        .as_ref()
        .and_then(|n| c.providers.get(n))
        .ok_or_else(|| anyhow::anyhow!("No provider selected"))?;
    if o.local {
        return Ok(());
    }
    if p.kind == "process" {
        output::emit(&format!(
            "Configured executable: {} (review arguments in config). It can access files and network under your account. Input: {context}",
            p.executable.as_deref().unwrap_or("unknown")
        ))?;
        o.allow_exec = yes("Execute this configured tool? Type yes: ")?;
    } else if !p
        .endpoint
        .as_ref()
        .and_then(|s| url::Url::parse(s).ok())
        .is_some_and(|u| config::is_loopback(&u))
    {
        output::emit(&format!(
            "Provider: {}\nEndpoint: {}\nTransfer: {context}",
            c.active.as_deref().unwrap_or("unknown"),
            p.endpoint.as_deref().unwrap_or("unknown")
        ))?;
        o.allow_external = yes("Authorize this request? Type yes: ")?;
    }
    Ok(())
}
fn connect(
    c: &mut Config,
    path: &Path,
    trusted: bool,
    local: bool,
    user_config: bool,
    max_output_tokens: Option<u32>,
) -> Result<()> {
    output::emit(
        "Types: api (compatible HTTPS service), local (literal loopback compatible service), process (installed executable; can access your account's files/network).\nAuthentication: API none or bearer token via a named environment variable; process tool manages its own login. No credentials are entered or stored here.",
    )?;
    let Some(name) = line("Provider name: ")? else {
        return Ok(());
    };
    if c.providers.contains_key(&name) {
        anyhow::bail!("Provider already exists; use /use or remove explicitly before replacing")
    }
    let Some(kind) = line("Type [api/local/process]: ")? else {
        return Ok(());
    };
    let mut p = config::ProviderConfig {
        kind: kind.clone(),
        endpoint: None,
        model: None,
        key_env: None,
        executable: None,
        args: vec![],
        timeout: 30,
    };
    if kind == "process" {
        p.executable = line("Executable (no arguments; use providers add for fixed arguments): ")?;
        if let Some(e) = &p.executable {
            output::emit(
                if Path::new(e).is_file() || project::executable(e).is_some() {
                    "Executable found; authentication and execution untested."
                } else {
                    "Executable not found; install it independently before use."
                },
            )?;
        }
    } else {
        p.endpoint = line("Base endpoint including /v1 if required: ")?;
        p.model = line("Model identifier: ")?;
        p.key_env = line("Bearer token environment variable name (blank for no authentication): ")?
            .filter(|s| !s.is_empty());
    }
    let mut updated = c.clone();
    updated.providers.insert(name.clone(), p);
    if user_config {
        updated.active = Some(name.clone());
    }
    updated.validate()?;
    if !yes("Save this configuration locally? Type yes: ")? {
        return output::emit("Configuration discarded.");
    }
    if user_config {
        let directory = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("User config has no parent directory"))?;
        std::fs::create_dir_all(directory)?;
    }
    config::save(path, &updated, !path.exists())?;
    *c = updated;
    c.active = Some(name);
    output::emit(
        "Saved and selected for this session. Connectivity untested. Use /models or send a prompt to test generation. The session's local-only policy remains enforced.",
    )?;
    if trusted
        && yes(
            "Test connectivity and text generation with the fixed prompt 'Reply with OK.'? Type yes: ",
        )?
    {
        let mut options = AiOptions {
            local,
            ..Default::default()
        };
        consent(
            c,
            &mut options,
            "fixed probe: Reply with OK.; no files or history",
        )?;
        let (_, provider) = providers::selected(c, &options, trusted)?;
        let result = providers::adapter(provider)
            .generate_with_limit("Reply with OK.", max_output_tokens)?;
        if result.finish_reason.as_deref() == Some("length") {
            return output::emit(
                "Provider responded, but the probe reached the token limit; text generation remains incomplete. Increase /budget if needed.",
            );
        }
        output::emit("Connectivity and text generation succeeded for this configured model.")?;
    }
    Ok(())
}
