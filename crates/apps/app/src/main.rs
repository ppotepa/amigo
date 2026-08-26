use amigo_app::BootstrapOptions;
use amigo_core::{AmigoError, AmigoResult};
use amigo_modding::requested_mods_for_root;

#[derive(Debug, Default)]
struct AppArgs {
    editor: bool,
    hosted: bool,
    dev: bool,
    mods_root: Option<String>,
    startup_mod: Option<String>,
    startup_scene: Option<String>,
    active_mods: Option<String>,
    help: bool,
}

fn main() -> AmigoResult<()> {
    let args = parse_args(std::env::args().skip(1).collect())?;
    if args.help {
        print_usage();
        return Ok(());
    }

    let editor_requested = args.editor;
    let hosted = args.hosted || editor_requested;
    let dev_mode = args.dev || editor_requested;
    let mods_root = args.mods_root.unwrap_or_else(|| "mods".to_owned());
    let startup_mod = args
        .startup_mod
        .or_else(|| editor_requested.then(|| "rotten-club".to_owned()));
    let startup_scene = args
        .startup_scene
        .or_else(|| editor_requested.then(|| "main-menu".to_owned()));
    let active_mods = args.active_mods.map(|mods| {
        mods.split(',')
            .filter(|mod_id| !mod_id.trim().is_empty())
            .map(|mod_id| mod_id.trim().to_owned())
            .collect::<Vec<_>>()
    });

    let mut options = BootstrapOptions::new(mods_root)
        .with_dev_mode(dev_mode)
        .with_editor_mode(editor_requested);

    if let Some(active_mods) = active_mods {
        options = options.with_active_mods(active_mods);
    }
    if let Some(startup_mod) = startup_mod {
        if options.active_mods.is_none() {
            options = options.with_active_mods(requested_mods_for_root(&startup_mod));
        }
        options = options.with_startup_mod(startup_mod);
    }
    if let Some(startup_scene) = startup_scene {
        options = options.with_startup_scene(startup_scene);
    }

    if hosted {
        amigo_app::run_hosted_with_options(options)?;
    } else {
        let bootstrap = amigo_app::bootstrap_session_with_options(options)?;
        println!("{}", bootstrap.summary());
    }

    Ok(())
}

fn parse_args(raw: Vec<String>) -> AmigoResult<AppArgs> {
    let mut parsed = AppArgs::default();
    let mut index = 0;
    while index < raw.len() {
        let argument = &raw[index];
        match argument.as_str() {
            "--editor" => parsed.editor = true,
            "--hosted" => parsed.hosted = true,
            "--dev" => parsed.dev = true,
            "--help" | "-h" => parsed.help = true,
            "--mods-root" | "--mod" | "--scene" | "--mods" => {
                let value = raw.get(index + 1).ok_or_else(|| {
                    AmigoError::Message(format!("missing value for `{argument}`"))
                })?;
                if value.starts_with("--") || value.is_empty() {
                    return Err(AmigoError::Message(format!(
                        "missing value for `{argument}`"
                    )));
                }
                set_option(&mut parsed, argument, value.clone())?;
                index += 1;
            }
            _ if argument.starts_with("--mods-root=")
                || argument.starts_with("--mod=")
                || argument.starts_with("--scene=")
                || argument.starts_with("--mods=") =>
            {
                let (flag, value) = argument
                    .split_once('=')
                    .expect("guarded option should contain equals");
                if value.is_empty() {
                    return Err(AmigoError::Message(format!("missing value for `{flag}`")));
                }
                set_option(&mut parsed, flag, value.to_owned())?;
            }
            _ => {
                return Err(AmigoError::Message(format!(
                    "unknown amigo-app argument `{argument}`; use --help for supported options"
                )));
            }
        }
        index += 1;
    }
    Ok(parsed)
}

fn set_option(parsed: &mut AppArgs, flag: &str, value: String) -> AmigoResult<()> {
    let slot = match flag {
        "--mods-root" => &mut parsed.mods_root,
        "--mod" => &mut parsed.startup_mod,
        "--scene" => &mut parsed.startup_scene,
        "--mods" => &mut parsed.active_mods,
        _ => unreachable!("validated option flag"),
    };
    if slot.is_some() {
        return Err(AmigoError::Message(format!(
            "option `{flag}` was provided more than once"
        )));
    }
    *slot = Some(value);
    Ok(())
}

fn print_usage() {
    println!("Amigo runtime host");
    println!("usage: amigo-app [--hosted] [--dev] [--editor] [--mods-root PATH] [--mod ID] [--scene ID] [--mods ID,ID]");
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    #[test]
    fn rejects_unknown_arguments() {
        assert!(parse_args(vec!["--typo".to_owned()]).is_err());
    }

    #[test]
    fn rejects_missing_option_values() {
        assert!(parse_args(vec!["--scene".to_owned()]).is_err());
    }

    #[test]
    fn accepts_equals_and_separate_option_values() {
        let args = parse_args(vec![
            "--hosted".to_owned(),
            "--mod=rotten-club".to_owned(),
            "--scene".to_owned(),
            "main-menu".to_owned(),
        ])
        .unwrap();
        assert!(args.hosted);
        assert_eq!(args.startup_mod.as_deref(), Some("rotten-club"));
        assert_eq!(args.startup_scene.as_deref(), Some("main-menu"));
    }
}
