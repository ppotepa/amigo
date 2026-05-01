use std::cmp::Ordering;
use std::collections::BTreeSet;

use super::{KnownMod, ModSceneManifest};

pub(super) fn mod_matches_filter(known_mod: &KnownMod, filter: &str) -> bool {
    let filter = normalize_filter_text(filter);
    if filter.is_empty() {
        return true;
    }

    let primary = normalize_filter_text(&format!("{} {}", known_mod.id, known_mod.name));
    let description = normalize_filter_text(&known_mod.description);

    primary.contains(&filter)
        || tokenize_filter_text(&primary)
            .into_iter()
            .any(|token| is_fuzzy_subsequence(&filter, &token))
        || description.contains(&filter)
}

pub(super) fn scene_matches_filter(scene: &ModSceneManifest, filter: &str) -> bool {
    let filter = normalize_filter_text(filter);
    if filter.is_empty() {
        return true;
    }

    let primary = normalize_filter_text(&format!("{} {}", scene.id, scene.label));
    let description = normalize_filter_text(&scene.description.clone().unwrap_or_default());

    primary.contains(&filter)
        || tokenize_filter_text(&primary)
            .into_iter()
            .any(|token| is_fuzzy_subsequence(&filter, &token))
        || description.contains(&filter)
}

pub(super) fn category_matches_filter(category: &[String], filter: &str) -> bool {
    let filter = normalize_filter_text(filter);
    if filter.is_empty() {
        return true;
    }
    let category_text = normalize_filter_text(&category.join(" "));
    category_text.contains(&filter)
        || tokenize_filter_text(&category_text)
            .into_iter()
            .any(|token| is_fuzzy_subsequence(&filter, &token))
}

pub(super) fn launcher_category_for_scene(known_mod: &KnownMod, scene: &ModSceneManifest) -> Vec<String> {
    if let Some(category) = known_mod.launcher_scene_categories.get(&scene.id) {
        return category.clone();
    }
    if let Some(category) = known_mod.launcher_category.as_ref() {
        return category.clone();
    }
    let segments = match known_mod.id.as_str() {
        "playground-2d" => match scene.id.as_str() {
            "hello-world-spritesheet" | "sprite-lab" => vec!["2D", "Sprites"],
            "text-lab" => vec!["2D", "Text"],
            "screen-space-preview" => vec!["UI", "HUD"],
            _ => vec!["2D", "Basics"],
        },
        "playground-2d-particles" => vec!["2D", "FX", "Particles"],
        "playground-sidescroller" => vec!["2D", "Games", "Sidescroller"],
        "playground-3d" => match scene.id.as_str() {
            "mesh-lab" => vec!["3D", "Meshes"],
            "material-lab" => vec!["3D", "Materials"],
            _ => vec!["3D", "Basics"],
        },
        "playground-hud-ui" => vec!["UI", "HUD"],
        "core-game" => vec!["Tools", "Dev"],
        "core" => vec!["Core"],
        _ => return launcher_category_for_mod(known_mod),
    };
    segments.into_iter().map(str::to_owned).collect()
}

pub(super) fn launcher_category_for_mod(known_mod: &KnownMod) -> Vec<String> {
    if let Some(category) = known_mod.launcher_category.as_ref() {
        return category.clone();
    }
    match known_mod.id.as_str() {
        "playground-2d" => vec!["2D", "Basics"],
        "playground-2d-particles" => vec!["2D", "FX", "Particles"],
        "playground-sidescroller" => vec!["2D", "Games", "Sidescroller"],
        "playground-3d" => vec!["3D", "Basics"],
        "playground-hud-ui" => vec!["UI", "HUD"],
        "core-game" => vec!["Tools", "Dev"],
        "core" => vec!["Core"],
        _ => vec!["Other"],
    }
    .into_iter()
    .map(str::to_owned)
    .collect()
}

pub(super) fn default_expanded_category_ids(known_mods: &[KnownMod]) -> BTreeSet<String> {
    let mut expanded = BTreeSet::new();
    for known_mod in known_mods {
        if known_mod.scenes.is_empty() {
            for prefix in category_prefixes(&launcher_category_for_mod(known_mod)) {
                expanded.insert(category_id(&prefix));
            }
            continue;
        }
        for scene in &known_mod.scenes {
            for prefix in category_prefixes(&launcher_category_for_scene(known_mod, scene)) {
                expanded.insert(category_id(&prefix));
            }
        }
    }
    expanded
}

pub(super) fn category_prefixes(category: &[String]) -> Vec<Vec<String>> {
    (1..=category.len())
        .map(|length| category[..length].to_vec())
        .collect()
}

pub(super) fn category_id(category: &[String]) -> String {
    category.join("/")
}

pub(super) fn mod_node_id(category_id: &str, mod_id: &str) -> String {
    format!("{category_id}::{mod_id}")
}

pub(super) fn compare_launcher_category_paths(left: &Vec<String>, right: &Vec<String>) -> Ordering {
    launcher_category_sort_key(left)
        .cmp(&launcher_category_sort_key(right))
        .then_with(|| left.cmp(right))
}

fn launcher_category_sort_key(path: &[String]) -> Vec<usize> {
    path.iter()
        .map(|segment| match segment.as_str() {
            "2D" => 0,
            "3D" => 10,
            "UI" => 20,
            "HUD" => 21,
            "Tools" => 30,
            "Dev" => 31,
            "Core" => 40,
            "Basics" => 1,
            "Sprites" => 2,
            "Text" => 3,
            "FX" => 4,
            "Particles" => 5,
            "Games" => 6,
            "Sidescroller" => 8,
            "Meshes" => 11,
            "Materials" => 12,
            _ => 99,
        })
        .collect()
}

pub(super) fn normalize_filter_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| character.to_lowercase())
        .collect()
}

fn tokenize_filter_text(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_owned())
        .collect()
}

pub(super) fn is_fuzzy_subsequence(needle: &str, haystack: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    let mut needle = needle.chars();
    let mut expected = needle.next();

    for character in haystack.chars() {
        if Some(character) == expected {
            expected = needle.next();
            if expected.is_none() {
                return true;
            }
        }
    }

    false
}

pub(super) fn is_scene_filter_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ' ' | '.')
}

pub(super) fn display_string_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_owned()
    } else {
        values.join(", ")
    }
}

pub(super) fn wrapped_next_index(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }

    (current as isize + delta).rem_euclid(len as isize) as usize
}
