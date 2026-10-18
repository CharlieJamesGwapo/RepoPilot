//! Offline, curated local model metadata and explicit configuration scaffolding.
use crate::{
    cli::ModelAction,
    config::{self, Config, ProviderConfig},
    output,
};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::{fs, path::Path};
#[derive(Serialize)]
struct Preset {
    id: &'static str,
    model: &'static str,
    approximate_weights: &'static str,
    license: &'static str,
    source: &'static str,
    runtime_source: &'static str,
    pull_command: &'static str,
    guidance: &'static str,
}
static PRESETS: &[Preset] = &[
    Preset {
        id: "light",
        model: "qwen2.5-coder:1.5b",
        approximate_weights: "986 MB",
        license: "Apache-2.0",
        source: "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct",
        runtime_source: "https://ollama.com/library/qwen2.5-coder:1.5b",
        pull_command: "ollama pull qwen2.5-coder:1.5b",
        guidance: "Smaller download for constrained systems; assess answer quality on your tasks.",
    },
    Preset {
        id: "balanced",
        model: "qwen2.5-coder:7b",
        approximate_weights: "4.7 GB",
        license: "Apache-2.0",
        source: "https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct",
        runtime_source: "https://ollama.com/library/qwen2.5-coder:7b",
        pull_command: "ollama pull qwen2.5-coder:7b",
        guidance: "Suggested starting point for a 16 GB Mac; opt-in download, with additional runtime/context memory required.",
    },
    Preset {
        id: "workstation",
        model: "qwen3-coder:30b",
        approximate_weights: "19 GB",
        license: "Apache-2.0",
        source: "https://huggingface.co/Qwen/Qwen3-Coder-30B-A3B-Instruct",
        runtime_source: "https://ollama.com/library/qwen3-coder:30b",
        pull_command: "ollama pull qwen3-coder:30b",
        guidance: "Larger workstation option; weights alone exceed 16 GB. Not recommended for a 16 GB Mac.",
    },
];
fn linked(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}
// The leaf may not exist yet: normalize the existing parent, then append its name.
fn canonical_leaf(path: &Path) -> Option<std::path::PathBuf> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Some(parent.canonicalize().ok()?.join(path.file_name()?))
}
fn same_location(a: &Path, b: &Path) -> bool {
    // Reject ASCII case variants of the reserved filename on every filesystem,
    // including case-sensitive volumes: keeping user config separate is preferable
    // to allowing an alias on default case-insensitive macOS/Windows volumes.
    let same_name = a.file_name().zip(b.file_name()).is_some_and(|(a, b)| {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    });
    #[cfg(any(windows, target_os = "macos"))]
    let same_parent = a.parent().zip(b.parent()).is_some_and(|(a, b)| {
        a.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
    });
    #[cfg(not(any(windows, target_os = "macos")))]
    let same_parent = a.parent() == b.parent();
    same_name && same_parent
}
fn check_output(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let meta = fs::symlink_metadata(parent).context("Output parent must already exist")?;
    if !meta.is_dir() || linked(&meta) {
        bail!("Output parent must be a regular directory, not a symlink")
    }
    match fs::symlink_metadata(path) {
        Ok(_) => bail!("Output already exists; nothing overwritten"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
pub fn run(action: &ModelAction, json: bool, quiet: bool) -> Result<()> {
    match action {
        ModelAction::List => {
            if json {
                output::json(&PRESETS)?
            } else if !quiet {
                output::emit(
                    "RepoPilot local model presets (offline catalog; no models installed or tested)",
                )?;
                for p in PRESETS {
                    output::emit(&format!(
                        "{}: {} (~{} weights; {})\n  {}\n  Source: {}\n  Runtime tag: {}\n  Manual download: {}",
                        p.id,
                        p.model,
                        p.approximate_weights,
                        p.license,
                        p.guidance,
                        p.source,
                        p.runtime_source,
                        p.pull_command
                    ))?;
                }
                output::emit(
                    "Weights are approximate downloads, not RAM requirements. Runtime and context require extra memory. Models can assist frontend, backend and DevOps; no role-specific fine-tuning or quality guarantee is implied. No downloads, service startup or network requests were performed.",
                )?;
            }
        }
        ModelAction::Setup {
            preset,
            output: path,
        } => {
            let p = PRESETS
                .iter()
                .find(|p| p.id == preset)
                .context("Unknown model preset; use repopilot models list")?;
            check_output(path)?;
            if config::user_path()
                .ok()
                .and_then(|user| canonical_leaf(&user))
                .zip(canonical_leaf(path))
                .is_some_and(|(user, chosen)| same_location(&user, &chosen))
            {
                bail!(
                    "Choose a separate explicit output file; saved user configuration is not changed"
                )
            }
            let mut c = Config {
                active: Some("local-coder".into()),
                ..Config::default()
            };
            c.providers.insert(
                "local-coder".into(),
                ProviderConfig {
                    kind: "local".into(),
                    endpoint: Some("http://127.0.0.1:11434/v1".into()),
                    model: Some(p.model.into()),
                    key_env: None,
                    executable: None,
                    args: vec![],
                    timeout: 180,
                },
            );
            config::save(path, &c, true)?;
            if json {
                output::json(
                    &serde_json::json!({"status":"created","output":path,"preset":p,"endpoint":"http://127.0.0.1:11434/v1","network_accessed":false,"downloaded":false,"note":"Run the pull command manually only if you want a model download; runtime/context need extra memory. No saved user configuration changed."}),
                )?
            } else if !quiet {
                output::emit(&format!(
                    "Created explicit local configuration: {:?}\nModel: {} (~{} weights; {})\n{}\nSource: {}\nRuntime tag: {}\nManual opt-in download: {}\nWith your compatible local runtime running: repopilot --config {:?} chat --local\nNo download, service startup or network request performed. Saved user configuration is unchanged. Runtime/context need extra memory; review generated code and commands before using them.",
                    path,
                    p.model,
                    p.approximate_weights,
                    p.license,
                    p.guidance,
                    p.source,
                    p.runtime_source,
                    p.pull_command,
                    path
                ))?;
            }
        }
    }
    Ok(())
}
