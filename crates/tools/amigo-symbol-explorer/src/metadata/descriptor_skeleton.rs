use anyhow::Result;

pub fn print_descriptor_skeleton(component_kind: &str) -> Result<()> {
    let function_name = to_snake_case(component_kind);
    println!("pub fn {function_name}_descriptor() -> ComponentTypeDescriptor {{");
    println!("    ComponentTypeDescriptor {{");
    println!("        kind: ComponentKind::{component_kind},");
    println!("        type_name: \"{component_kind}\",");
    println!("        label: \"{component_kind}\",");
    println!("        domains: &[ComponentDomain::Data],");
    println!("        capabilities: &[],");
    println!("        asset_refs: &[],");
    println!("        transform_policy: TransformPolicy::None,");
    println!("        bounds_policy: BoundsPolicy::None,");
    println!("        editor_controls: &[],");
    println!("        patch_ops: &[],");
    println!("    }}");
    println!("}}");
    Ok(())
}

fn to_snake_case(value: &str) -> String {
    let mut out = String::new();
    let mut previous: Option<char> = None;
    for ch in value.chars() {
        let needs_separator = previous.is_some_and(|prev| {
            (ch.is_ascii_uppercase() && !prev.is_ascii_uppercase() && !prev.is_ascii_digit())
                || (ch.is_ascii_digit() && !prev.is_ascii_digit())
                || (!ch.is_ascii_digit() && !ch.is_ascii_uppercase() && prev.is_ascii_digit())
        });
        if needs_separator {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
        previous = Some(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::to_snake_case;

    #[test]
    fn maps_component_kind_to_descriptor_stem() {
        assert_eq!(to_snake_case("AabbCollider2D"), "aabb_collider_2d");
        assert_eq!(to_snake_case("Sprite2D"), "sprite_2d");
        assert_eq!(to_snake_case("UiDocument"), "ui_document");
    }
}
