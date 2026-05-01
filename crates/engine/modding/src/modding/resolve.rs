pub fn requested_mods_for_root(root_mod_id: &str) -> Vec<String> {
    if root_mod_id == CORE_MOD_ID {
        vec![CORE_MOD_ID.to_owned()]
    } else {
        vec![CORE_MOD_ID.to_owned(), root_mod_id.to_owned()]
    }
}

fn discover_mod_map(mods_root: &Path) -> AmigoResult<BTreeMap<String, DiscoveredMod>> {
    if !mods_root.exists() {
        return Ok(BTreeMap::new());
    }

    let mut discovered = BTreeMap::new();

    for entry in fs::read_dir(mods_root)? {
        let entry = entry?;
        let mod_root = entry.path();

        if !mod_root.is_dir() {
            continue;
        }

        let manifest_path = mod_root.join("mod.toml");

        if !manifest_path.is_file() {
            continue;
        }

        let raw = fs::read_to_string(&manifest_path)?;
        let manifest = toml::from_str::<ModManifest>(&raw)
            .map_err(|error| AmigoError::Message(error.to_string()))?;
        let mod_id = manifest.id.clone();

        if discovered
            .insert(
                mod_id.clone(),
                DiscoveredMod {
                    manifest,
                    root_path: mod_root,
                },
            )
            .is_some()
        {
            return Err(AmigoError::Message(format!(
                "duplicate mod id `{mod_id}` found under `{}`",
                mods_root.display()
            )));
        }
    }

    Ok(discovered)
}

fn resolve_discovered_mods(
    discovered: &BTreeMap<String, DiscoveredMod>,
    selected_mod_ids: &[String],
) -> AmigoResult<Vec<DiscoveredMod>> {
    let mut ordered = Vec::new();
    let mut visited = BTreeSet::new();
    let mut visiting = BTreeSet::new();

    for mod_id in selected_mod_ids {
        resolve_mod(
            mod_id,
            None,
            discovered,
            &mut visiting,
            &mut visited,
            &mut ordered,
        )?;
    }

    Ok(ordered)
}

fn resolve_mod(
    mod_id: &str,
    requested_by: Option<&str>,
    discovered: &BTreeMap<String, DiscoveredMod>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    ordered: &mut Vec<DiscoveredMod>,
) -> AmigoResult<()> {
    if visited.contains(mod_id) {
        return Ok(());
    }

    if !visiting.insert(mod_id.to_owned()) {
        return Err(AmigoError::Message(format!(
            "dependency cycle detected while resolving mod `{mod_id}`"
        )));
    }

    let discovered_mod = discovered.get(mod_id).ok_or_else(|| {
        let message = match requested_by {
            Some(parent_mod_id) => {
                format!("mod `{parent_mod_id}` depends on missing mod `{mod_id}`")
            }
            None => format!("configured mod `{mod_id}` was not found in the discovered catalog"),
        };
        AmigoError::Message(message)
    })?;

    for dependency_id in discovered_mod.manifest.dependencies.clone() {
        resolve_mod(
            &dependency_id,
            Some(mod_id),
            discovered,
            visiting,
            visited,
            ordered,
        )?;
    }

    visiting.remove(mod_id);

    if visited.insert(mod_id.to_owned()) {
        ordered.push(discovered_mod.clone());
    }

    Ok(())
}

