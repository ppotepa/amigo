fn property_visible_for_view(row: &AuthoringProperty, mode: InspectorViewMode) -> bool {
    match mode {
        InspectorViewMode::Primary => {
            matches!(row.display.visibility, AuthoringPropertyVisibility::Primary)
        }
        InspectorViewMode::Advanced => {
            !matches!(row.display.visibility, AuthoringPropertyVisibility::Hidden)
        }
        InspectorViewMode::RawDebug => true,
    }
}

fn display_for_binding(
    binding: &Option<AuthoringRuntimeBinding>,
    read_only: bool,
    visibility: AuthoringPropertyVisibility,
    mut tags: Vec<String>,
) -> AuthoringPropertyDisplay {
    let apply_mode = if read_only {
        AuthoringPropertyApplyMode::ReadOnly
    } else {
        match binding {
            Some(AuthoringRuntimeBinding::Mock { .. })
            | Some(AuthoringRuntimeBinding::PostFxMock { .. }) => AuthoringPropertyApplyMode::Mock,
            Some(_) => AuthoringPropertyApplyMode::Live,
            None => AuthoringPropertyApplyMode::Unsupported,
        }
    };

    match apply_mode {
        AuthoringPropertyApplyMode::Live => tags.push("Live".to_owned()),
        AuthoringPropertyApplyMode::Mock => tags.push("Mock".to_owned()),
        AuthoringPropertyApplyMode::ReadOnly => tags.push("Readonly".to_owned()),
        AuthoringPropertyApplyMode::Unsupported => tags.push("Unsupported".to_owned()),
    }

    AuthoringPropertyDisplay {
        icon: None,
        tags,
        visibility,
        apply_mode,
        order: 0,
    }
}

fn authoring_visibility(visibility: ScenePropertyVisibility) -> AuthoringPropertyVisibility {
    match visibility {
        ScenePropertyVisibility::Primary => AuthoringPropertyVisibility::Primary,
        ScenePropertyVisibility::Advanced => AuthoringPropertyVisibility::Advanced,
        ScenePropertyVisibility::Debug => AuthoringPropertyVisibility::Debug,
        ScenePropertyVisibility::Hidden => AuthoringPropertyVisibility::Hidden,
    }
}

