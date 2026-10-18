use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub max_files: usize,
    pub max_depth: usize,
    pub exclude: Vec<String>,
    pub active: Option<String>,
    pub providers: BTreeMap<String, ProviderConfig>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            max_files: 20_000,
            max_depth: 24,
            exclude: vec![],
            active: None,
            providers: BTreeMap::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderConfig {
    pub kind: String,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub key_env: Option<String>,
    pub executable: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "timeout")]
    pub timeout: u64,
}
fn timeout() -> u64 {
    30
}
pub fn path(explicit: &Option<PathBuf>) -> PathBuf {
    explicit
        .clone()
        .unwrap_or_else(|| PathBuf::from("repopilot.toml"))
}
/// User-owned chat configuration; never resolve a relative path against a repository.
pub fn user_path() -> Result<PathBuf> {
    #[cfg(not(windows))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|v| !v.is_empty())
                .map(|p| PathBuf::from(p).join(".config"))
        });
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE")
                .filter(|v| !v.is_empty())
                .map(|p| PathBuf::from(p).join(".config"))
        });
    let base = base
        .context("Cannot locate user configuration directory; supply --config PATH explicitly")?;
    if !base.is_absolute() {
        bail!(
            "User configuration directory must be absolute; supply an absolute XDG_CONFIG_HOME, home directory, or --config PATH"
        )
    }
    Ok(base.join("repopilot/config.toml"))
}
pub fn load(p: &Path, required: bool) -> Result<Config> {
    if !p.exists() && !required {
        return Ok(Config::default());
    }
    let meta =
        fs::symlink_metadata(p).context("Cannot inspect configuration; check the supplied path")?;
    if meta.file_type().is_symlink() || !meta.is_file() || meta.len() > 262144 {
        bail!("Configuration must be a regular file of at most 256 KiB (no symlinks)")
    }
    let text = crate::project::bounded_text(p, 262144)
        .context("Cannot read bounded configuration as UTF-8")?;
    let c:Config=toml::from_str(&text).map_err(|_|anyhow::anyhow!("Invalid configuration TOML or unknown fields; inspect local configuration (contents omitted)"))?;
    c.validate()?;
    Ok(c)
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.max_files == 0
            || self.max_files > 1_000_000
            || self.max_depth == 0
            || self.max_depth > 128
        {
            bail!("Traversal limits: max_files must be 1..1000000 and max_depth 1..128")
        }
        if self
            .exclude
            .iter()
            .any(|s| s.is_empty() || s.contains('/') || s.contains('\\'))
        {
            bail!("exclude entries must be directory basenames")
        }
        for (name, p) in &self.providers {
            if name.is_empty()
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                bail!("Provider names must contain only ASCII letters, digits, '-' or '_'")
            }
            if !(1..=300).contains(&p.timeout) {
                bail!("Provider timeout must be 1..300 seconds")
            }
            if let Some(key) = &p.key_env
                && (key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
            {
                bail!("key_env must name an environment variable; raw keys are unsupported")
            }
            // Validate original text before URL parsing can normalize controls away.
            // These metadata fields can appear in terminal consent and status output.
            for (field, value) in [
                ("endpoint", &p.endpoint),
                ("model", &p.model),
                ("executable", &p.executable),
            ] {
                if value
                    .as_deref()
                    .is_some_and(|text| text.chars().any(char::is_control))
                {
                    bail!("Provider {field} must not contain control characters");
                }
            }
            match p.kind.as_str() {
                "api" | "local" => {
                    let u = url::Url::parse(p.endpoint.as_deref().unwrap_or("")).map_err(|_| {
                        anyhow::anyhow!("Provider endpoint must be an absolute HTTP(S) URL")
                    })?;
                    if !u.username().is_empty()
                        || u.password().is_some()
                        || u.query().is_some()
                        || u.fragment().is_some()
                    {
                        bail!(
                            "Endpoint must not contain credentials, query parameters or fragments"
                        )
                    }
                    if !matches!(u.scheme(), "http" | "https") || u.host_str().is_none() {
                        bail!("Endpoint must use HTTP(S)")
                    }
                    let local = is_loopback(&u);
                    if p.kind == "local" && !local {
                        bail!(
                            "local providers must use literal loopback addresses (127.0.0.1 or [::1])"
                        )
                    }
                    if !local && u.scheme() != "https" {
                        bail!("Remote providers require HTTPS")
                    }
                    if p.model.as_deref().is_none_or(str::is_empty) {
                        bail!("API providers require a model")
                    }
                    if p.executable.is_some() || !p.args.is_empty() {
                        bail!("API providers cannot declare executables or arguments")
                    }
                }
                "process" => {
                    if p.executable.as_deref().is_none_or(str::is_empty) {
                        bail!("Process providers require an executable")
                    }
                    if p.endpoint.is_some() || p.key_env.is_some() {
                        bail!(
                            "Process adapters do not accept endpoints or key_env; configure the installed tool independently"
                        )
                    }
                }
                _ => bail!("Unsupported provider kind; choose api, local or process"),
            }
        }
        if let Some(a) = &self.active
            && !self.providers.contains_key(a)
        {
            bail!("Active provider is not configured")
        }
        Ok(())
    }
}
pub fn is_loopback(u: &url::Url) -> bool {
    match u.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        _ => false,
    }
}
pub fn save(p: &Path, c: &Config, create: bool) -> Result<()> {
    c.validate()?;
    if p.exists() && fs::symlink_metadata(p)?.file_type().is_symlink() {
        bail!("Refusing to write a symlink configuration")
    }
    if !create && !p.is_file() {
        bail!("Existing configuration must be a regular file")
    }
    let parent = p
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut f = tempfile::NamedTempFile::new_in(parent)
        .context("Cannot create configuration in this directory")?;
    f.write_all(b"# RepoPilot: no secrets here. key_env names an environment variable.\n# exclude lists directory basenames; traversal never follows symlinks.\n")?;
    f.write_all(toml::to_string_pretty(c)?.as_bytes())?;
    f.as_file().sync_all()?;
    if create {
        f.persist_noclobber(p)
    } else {
        f.persist(p)
    }
    .map_err(|_| {
        anyhow::anyhow!("Cannot save configuration; init refuses to overwrite existing files")
    })?;
    Ok(())
}
