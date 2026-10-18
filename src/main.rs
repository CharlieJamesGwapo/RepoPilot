mod chat;
mod cli;
mod config;
mod guides;
mod local_models;
mod output;
mod project;
mod providers;
mod shell;
mod skills;
mod welcome;
use anyhow::{Result, bail};
use clap::{CommandFactory, Parser};
use cli::{Cli, Cmd, ConfigAction, Format, ProviderAction};
fn execute(cli: Cli) -> Result<i32> {
    if let Some(Cmd::Models { ref action }) = cli.command {
        local_models::run(action, cli.json, cli.quiet)?;
        return Ok(0);
    }
    if let Some(Cmd::Skills { ref action }) = cli.command {
        skills::run(action, cli.json, cli.quiet)?;
        return Ok(0);
    }
    if let Some(Cmd::Guide { ref task }) = cli.command {
        guides::run(task, cli.json, cli.quiet)?;
        return Ok(0);
    }
    if matches!(cli.command, Some(Cmd::Welcome)) {
        if cli.json {
            bail!("welcome is a text screen; omit --json")
        }
        if !cli.quiet {
            output::emit(&welcome::render(welcome::stdout_is_terminal()))?;
        }
        return Ok(0);
    }
    let mut path = config::path(&cli.config);
    let explicit = cli.config.is_some();
    // Shell launch never reads repository/provider configuration or consumes stdin.
    if cli.command.is_none() || matches!(cli.command, Some(Cmd::Shell { .. })) {
        if cli.json || explicit {
            bail!(
                "shell mode does not use --json or --config; select a subcommand for reports or providers"
            )
        }
        let plain_prompt = matches!(cli.command, Some(Cmd::Shell { plain_prompt: true }));
        return shell::run(cli.quiet, plain_prompt);
    }
    if let Some(Cmd::Completions { shell }) = cli.command {
        clap_complete::generate(
            shell,
            &mut Cli::command(),
            "repopilot",
            &mut std::io::stdout(),
        );
        return Ok(0);
    }
    if let Some(Cmd::Rules) = cli.command {
        if cli.json {
            output::json(&project::RULES)?
        } else {
            for (id, text) in project::RULES {
                output::emit(&format!("{id}: {text}"))?
            }
        }
        return Ok(0);
    }
    if let Some(Cmd::Config {
        action: Some(ConfigAction::Init),
    }) = cli.command
    {
        config::save(&path, &config::Config::default(), true)?;
        if !cli.quiet {
            output::emit("Created starter configuration. No credentials stored.")?
        }
        return Ok(0);
    }
    let user_chat = matches!(cli.command, Some(Cmd::Chat { .. })) && !explicit;
    if user_chat {
        path = config::user_path()?;
    }
    let mut c = config::load(&path, explicit)?;
    if cli.verbose {
        eprintln!(
            "RepoPilot: local configuration validated; no connectivity probes run implicitly"
        );
    }
    let command = cli.command.expect("command checked above");
    let (root, format, doctor) = match &command {
        Cmd::Scan(a) | Cmd::Explain(a) => (Some(&a.path), Format::Terminal, false),
        Cmd::Doctor(a) | Cmd::Check(a) => (Some(&a.path), Format::Terminal, true),
        Cmd::Report { path, format } => (Some(path), *format, true),
        _ => (None, Format::Terminal, false),
    };
    if let Some(root) = root {
        let r = project::scan(root, &c, doctor)?;
        let code = i32::from(r.findings.iter().any(|f| f.severity == "error"));
        if cli.json || matches!(format, Format::Json) {
            output::json(&r)?
        } else if !cli.quiet {
            output::render(&r, format)?
        }
        return Ok(code);
    }
    match command {
        Cmd::Config { .. } => {
            if !cli.quiet || cli.json {
                let mut visible = c.clone();
                for p in visible.providers.values_mut() {
                    if !p.args.is_empty() {
                        p.args = vec![
                            "[arguments omitted; inspect trusted configuration locally]".into(),
                        ];
                    }
                }
                output::json(&visible)?
            }
        }
        Cmd::Providers { action } => match action {
            ProviderAction::List => output::json(&providers::list(&c))?,
            ProviderAction::Add {
                name,
                kind,
                endpoint,
                model,
                key_env,
                executable,
                args,
                timeout,
            } => {
                if c.providers.contains_key(&name) {
                    bail!("Provider already exists; remove explicitly before replacing")
                }
                c.providers.insert(
                    name,
                    config::ProviderConfig {
                        kind,
                        endpoint,
                        model,
                        key_env,
                        executable,
                        args,
                        timeout,
                    },
                );
                config::save(&path, &c, !path.exists())?;
                if !cli.quiet {
                    output::emit(
                        "Provider saved. Connectivity, authentication and task execution remain untested.",
                    )?
                }
            }
            ProviderAction::Use { name } => {
                if !c.providers.contains_key(&name) {
                    bail!("Provider is not configured")
                };
                c.active = Some(name);
                config::save(&path, &c, false)?;
                if !cli.quiet {
                    output::emit("Provider selected; no connectivity request performed.")?
                }
            }
            ProviderAction::Remove { name, yes } => {
                if !yes {
                    bail!(
                        "Removal requires explicit --yes confirmation; credentials in the environment are not removed"
                    )
                };
                if c.providers.remove(&name).is_none() {
                    bail!("Provider is not configured")
                };
                if c.active.as_ref() == Some(&name) {
                    c.active = None
                };
                config::save(&path, &c, false)?;
                if !cli.quiet {
                    output::emit("Provider removed. Environment credentials unchanged.")?
                }
            }
            ProviderAction::Models { options } => {
                if options.max_output_tokens.is_some() {
                    bail!("--max-output-tokens applies to generation, not model discovery");
                }
                let (_, p) = providers::selected(&c, &options, explicit)?;
                output::json(&providers::adapter(p).models()?)?
            }
            ProviderAction::Test { options } => {
                let (name, p) = providers::selected(&c, &options, explicit)?;
                let _ = providers::adapter(p)
                    .generate_with_limit("Reply with OK.", options.max_output_tokens)?;
                output::json(
                    &serde_json::json!({"provider":name,"connectivity":"success","text_generation":"success","model":p.model}),
                )?
            }
        },
        Cmd::Ask { prompt, options } => {
            let (name, p) = providers::selected(&c, &options, explicit)?;
            if cli.verbose {
                eprintln!(
                    "RepoPilot: selected provider {name}; typed prompt only; no files or history"
                );
            }
            let generation =
                providers::adapter(p).generate_with_limit(&prompt, options.max_output_tokens)?;
            let answer = &generation.text;
            if cli.verbose {
                eprintln!(
                    "RepoPilot usage (provider-reported): {}",
                    serde_json::to_string(&generation.usage)?
                );
            }
            if cli.json {
                output::json(
                    &serde_json::json!({"provider":name,"answer":answer,"usage":generation.usage,"finish_reason":generation.finish_reason,"output_limit":options.max_output_tokens}),
                )?
            } else if !cli.quiet {
                output::emit(answer)?;
                if generation.finish_reason.as_deref() == Some("length") {
                    eprintln!(
                        "RepoPilot: response reached the provider's token limit; adjust --max-output-tokens if needed"
                    );
                }
            }
        }
        Cmd::Chat {
            local,
            max_output_tokens,
        } => {
            if cli.json || cli.quiet {
                bail!("chat requires interactive text output; omit --json and --quiet")
            };
            chat::run(c, &path, explicit, local, user_chat, max_output_tokens)?
        }
        _ => unreachable!("project commands handled above"),
    }
    Ok(0)
}
fn main() {
    let code = match execute(Cli::parse()) {
        Ok(code) => code,
        Err(e) => {
            if e.downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::BrokenPipe)
            {
                return;
            }
            eprintln!("RepoPilot: {e:#}");
            2
        }
    };
    std::process::exit(code)
}
