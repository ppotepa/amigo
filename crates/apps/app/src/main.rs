use amigo_app::BootstrapOptions;
use amigo_core::AmigoResult;
use amigo_modding::requested_mods_for_root;

fn main() -> AmigoResult<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let hosted = has_flag(&args, "--hosted");
    let dev_mode = has_flag(&args, "--dev");
    let mods_root = parse_option_value(&args, "--mods-root").unwrap_or_else(|| "mods".to_owned());
    let startup_mod = parse_option_value(&args, "--mod");
    let startup_scene = parse_option_value(&args, "--scene");
    let active_mods = parse_option_value(&args, "--mods").map(|mods| {
        mods.split(',')
            .filter(|mod_id| !mod_id.trim().is_empty())
            .map(|mod_id| mod_id.trim().to_owned())
            .collect::<Vec<_>>()
    });

    let mut options = BootstrapOptions::new(mods_root).with_dev_mode(dev_mode);

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
    };

    if hosted {
        amigo_app::run_hosted_with_options(options)?;
    } else {
        let summary = amigo_app::run_with_options(options)?;
        println!("{summary}");
    }

    Ok(())
}

fn parse_option_value(args: &[String], flag: &str) -> Option<String> {
    for argument in args {
        if let Some(value) = argument.strip_prefix(&format!("{flag}=")) {
            return Some(value.to_owned());
        }
    }

    args.windows(2)
        .find_map(|window| (window[0] == flag).then(|| window[1].clone()))
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|argument| argument == flag)
}
