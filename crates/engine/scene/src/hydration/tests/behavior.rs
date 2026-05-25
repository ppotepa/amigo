fn plugin_payload<T: 'static>(command: &crate::SceneCommand) -> Option<&T> {
    match command {
        crate::SceneCommand::Plugin { command } => command.payload_as::<T>(),
        _ => None,
    }
}

mod sidescroller {
    include!("behavior/sidescroller.rs");
}

mod motion {
    include!("behavior/motion.rs");
}

mod particles {
    include!("behavior/particles.rs");
}

mod ui_theme {
    include!("behavior/ui_theme.rs");
}

mod projectile_camera {
    include!("behavior/projectile_camera.rs");
}

mod menu_state {
    include!("behavior/menu_state.rs");
}

mod transitions {
    include!("behavior/transitions.rs");
}
