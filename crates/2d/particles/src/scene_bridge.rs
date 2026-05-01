use crate::model::{
    ParticleAlignMode2d, ParticleBlendMode2d, ParticleEmitter2d, ParticleForce2d,
    ParticleLightMode2d, ParticleLineAnchor2d, ParticleShape2d, ParticleSimulationSpace2d,
    ParticleSpawnArea2d, ParticleVelocityMode2d,
};
use amigo_fx::ColorInterpolation;
use amigo_math::{ColorRgba, Curve1d};

pub fn particle_emitter_to_scene_yaml(emitter: &ParticleEmitter2d) -> String {
    let mut yaml = String::new();
    yaml.push_str("type: ParticleEmitter2D\n");
    if let Some(attached_to) = emitter.attached_to.as_deref() {
        yaml.push_str(&format!("attached_to: {attached_to}\n"));
    }
    yaml.push_str(&format!(
        "local_offset: {{ x: {}, y: {} }}\n",
        fmt_f32(emitter.local_offset.x),
        fmt_f32(emitter.local_offset.y)
    ));
    yaml.push_str(&format!(
        "local_direction_degrees: {}\n",
        fmt_f32(emitter.local_direction_radians.to_degrees())
    ));
    yaml.push_str(&format!("active: {}\n", emitter.active));
    yaml.push_str(&format!("spawn_rate: {}\n", fmt_f32(emitter.spawn_rate)));
    yaml.push_str(&format!("max_particles: {}\n", emitter.max_particles));
    yaml.push_str(&format!(
        "particle_lifetime: {}\n",
        fmt_f32(emitter.particle_lifetime)
    ));
    yaml.push_str(&format!(
        "lifetime_jitter: {}\n",
        fmt_f32(emitter.lifetime_jitter)
    ));
    yaml.push_str(&format!(
        "initial_speed: {}\n",
        fmt_f32(emitter.initial_speed)
    ));
    yaml.push_str(&format!(
        "speed_jitter: {}\n",
        fmt_f32(emitter.speed_jitter)
    ));
    yaml.push_str(&format!(
        "spread_degrees: {}\n",
        fmt_f32(emitter.spread_radians.to_degrees())
    ));
    yaml.push_str(&format!(
        "inherit_parent_velocity: {}\n",
        fmt_f32(emitter.inherit_parent_velocity)
    ));
    if emitter.velocity_mode != ParticleVelocityMode2d::Free {
        yaml.push_str(&format!(
            "velocity_mode: {}\n",
            velocity_mode_name(emitter.velocity_mode)
        ));
    }
    if emitter.simulation_space != ParticleSimulationSpace2d::World {
        yaml.push_str(&format!(
            "simulation_space: {}\n",
            simulation_space_name(emitter.simulation_space)
        ));
    }
    yaml.push_str(&format!(
        "initial_size: {}\n",
        fmt_f32(emitter.initial_size)
    ));
    yaml.push_str(&format!("final_size: {}\n", fmt_f32(emitter.final_size)));
    yaml.push_str(&format!("color: \"{}\"\n", color_to_hex(emitter.color)));
    yaml.push_str(&format!("z_index: {}\n", fmt_f32(emitter.z_index)));
    if let Some(color_ramp) = emitter.color_ramp.as_ref() {
        yaml.push_str("color_ramp:\n");
        yaml.push_str(&format!(
            "  interpolation: {}\n",
            color_interpolation_name(color_ramp.interpolation)
        ));
        yaml.push_str("  stops:\n");
        for stop in &color_ramp.stops {
            yaml.push_str(&format!(
                "    - {{ t: {}, color: \"{}\" }}\n",
                fmt_f32(stop.t),
                color_to_hex(stop.color)
            ));
        }
    }
    yaml.push_str("spawn_area:\n");
    append_spawn_area_yaml(&mut yaml, emitter.spawn_area, "  ");
    yaml.push_str("shape:\n");
    append_shape_yaml(&mut yaml, emitter.shape, "  ");
    if !emitter.shape_choices.is_empty() {
        yaml.push_str("shape_choices:\n");
        for choice in &emitter.shape_choices {
            yaml.push_str(&format!("  - weight: {}\n", fmt_f32(choice.weight)));
            yaml.push_str("    shape:\n");
            append_shape_yaml(&mut yaml, choice.shape, "      ");
        }
    }
    if !emitter.shape_over_lifetime.is_empty() {
        yaml.push_str("shape_over_lifetime:\n");
        for keyframe in &emitter.shape_over_lifetime {
            yaml.push_str(&format!("  - t: {}\n", fmt_f32(keyframe.t)));
            yaml.push_str("    shape:\n");
            append_shape_yaml(&mut yaml, keyframe.shape, "      ");
        }
    }
    yaml.push_str(&format!(
        "line_anchor: {}\n",
        line_anchor_name(emitter.line_anchor)
    ));
    yaml.push_str(&format!("align: {}\n", align_name(emitter.align)));
    yaml.push_str(&format!(
        "blend_mode: {}\n",
        blend_mode_name(emitter.blend_mode)
    ));
    if let Some(stretch) = emitter.motion_stretch {
        yaml.push_str("motion_stretch:\n");
        yaml.push_str(&format!("  enabled: {}\n", stretch.enabled));
        yaml.push_str(&format!(
            "  velocity_scale: {}\n",
            fmt_f32(stretch.velocity_scale)
        ));
        yaml.push_str(&format!("  max_length: {}\n", fmt_f32(stretch.max_length)));
    }
    if emitter.material.receives_light || (emitter.material.light_response - 1.0).abs() > 0.001 {
        yaml.push_str("material:\n");
        yaml.push_str(&format!(
            "  receives_light: {}\n",
            emitter.material.receives_light
        ));
        yaml.push_str(&format!(
            "  light_response: {}\n",
            fmt_f32(emitter.material.light_response)
        ));
    }
    if let Some(light) = emitter.light {
        yaml.push_str("light:\n");
        yaml.push_str(&format!("  radius: {}\n", fmt_f32(light.radius)));
        yaml.push_str(&format!("  intensity: {}\n", fmt_f32(light.intensity)));
        yaml.push_str(&format!("  mode: {}\n", light_mode_name(light.mode)));
        yaml.push_str(&format!("  glow: {}\n", light.glow));
    }
    if !emitter.forces.is_empty() {
        yaml.push_str("forces:\n");
        for force in &emitter.forces {
            append_force_yaml(&mut yaml, *force, "  ");
        }
    }
    yaml.push_str(&format!(
        "emission_rate_curve: {}\n",
        inline_curve_yaml(&emitter.emission_rate_curve)
    ));
    yaml.push_str(&format!(
        "size_curve: {}\n",
        inline_curve_yaml(&emitter.size_curve)
    ));
    yaml.push_str(&format!(
        "alpha_curve: {}\n",
        inline_curve_yaml(&emitter.alpha_curve)
    ));
    yaml.push_str(&format!(
        "speed_curve: {}\n",
        inline_curve_yaml(&emitter.speed_curve)
    ));
    yaml
}

fn append_spawn_area_yaml(yaml: &mut String, spawn_area: ParticleSpawnArea2d, indent: &str) {
    match spawn_area {
        ParticleSpawnArea2d::Point => yaml.push_str(&format!("{indent}kind: point\n")),
        ParticleSpawnArea2d::Line { length } => yaml.push_str(&format!(
            "{indent}kind: line\n{indent}length: {}\n",
            fmt_f32(length)
        )),
        ParticleSpawnArea2d::Rect { size } => yaml.push_str(&format!(
            "{indent}kind: rect\n{indent}size: {{ x: {}, y: {} }}\n",
            fmt_f32(size.x),
            fmt_f32(size.y)
        )),
        ParticleSpawnArea2d::Circle { radius } => yaml.push_str(&format!(
            "{indent}kind: circle\n{indent}radius: {}\n",
            fmt_f32(radius)
        )),
        ParticleSpawnArea2d::Ring {
            inner_radius,
            outer_radius,
        } => yaml.push_str(&format!(
            "{indent}kind: ring\n{indent}inner_radius: {}\n{indent}outer_radius: {}\n",
            fmt_f32(inner_radius),
            fmt_f32(outer_radius)
        )),
    }
}

fn append_shape_yaml(yaml: &mut String, shape: ParticleShape2d, indent: &str) {
    match shape {
        ParticleShape2d::Circle { segments } => yaml.push_str(&format!(
            "{indent}kind: circle\n{indent}segments: {segments}\n"
        )),
        ParticleShape2d::Quad => yaml.push_str(&format!("{indent}kind: quad\n")),
        ParticleShape2d::Line { length } => yaml.push_str(&format!(
            "{indent}kind: line\n{indent}length: {}\n",
            fmt_f32(length)
        )),
    }
}

fn append_force_yaml(yaml: &mut String, force: ParticleForce2d, indent: &str) {
    match force {
        ParticleForce2d::Gravity { acceleration } => yaml.push_str(&format!(
            "{indent}- kind: gravity\n{indent}  acceleration: {{ x: {}, y: {} }}\n",
            fmt_f32(acceleration.x),
            fmt_f32(acceleration.y)
        )),
        ParticleForce2d::ConstantAcceleration { acceleration } => yaml.push_str(&format!(
            "{indent}- kind: constant_acceleration\n{indent}  acceleration: {{ x: {}, y: {} }}\n",
            fmt_f32(acceleration.x),
            fmt_f32(acceleration.y)
        )),
        ParticleForce2d::Drag { coefficient } => yaml.push_str(&format!(
            "{indent}- kind: drag\n{indent}  coefficient: {}\n",
            fmt_f32(coefficient)
        )),
        ParticleForce2d::Wind { velocity, strength } => yaml.push_str(&format!(
            "{indent}- kind: wind\n{indent}  velocity: {{ x: {}, y: {} }}\n{indent}  strength: {}\n",
            fmt_f32(velocity.x),
            fmt_f32(velocity.y),
            fmt_f32(strength)
        )),
    }
}

fn velocity_mode_name(velocity_mode: ParticleVelocityMode2d) -> &'static str {
    match velocity_mode {
        ParticleVelocityMode2d::Free => "free",
        ParticleVelocityMode2d::SourceInertial => "source_inertial",
    }
}

fn simulation_space_name(simulation_space: ParticleSimulationSpace2d) -> &'static str {
    match simulation_space {
        ParticleSimulationSpace2d::World => "world",
        ParticleSimulationSpace2d::Source => "source",
    }
}

fn inline_curve_yaml(curve: &Curve1d) -> String {
    match curve {
        Curve1d::Constant(value) => format!("{{ kind: constant, value: {} }}", fmt_f32(*value)),
        Curve1d::Linear => "{ kind: linear }".to_owned(),
        Curve1d::EaseIn => "{ kind: ease_in }".to_owned(),
        Curve1d::EaseOut => "{ kind: ease_out }".to_owned(),
        Curve1d::EaseInOut => "{ kind: ease_in_out }".to_owned(),
        Curve1d::SmoothStep => "{ kind: smooth_step }".to_owned(),
        Curve1d::Custom { points } => {
            let points = points
                .iter()
                .map(|point| {
                    format!(
                        "{{ t: {}, value: {} }}",
                        fmt_f32(point.t),
                        fmt_f32(point.value)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ kind: custom, points: [{points}] }}")
        }
    }
}

fn color_interpolation_name(interpolation: ColorInterpolation) -> &'static str {
    match interpolation {
        ColorInterpolation::LinearRgb => "linear_rgb",
        ColorInterpolation::Step => "step",
    }
}

fn align_name(align: ParticleAlignMode2d) -> &'static str {
    match align {
        ParticleAlignMode2d::None => "none",
        ParticleAlignMode2d::Velocity => "velocity",
        ParticleAlignMode2d::Emitter => "emitter",
        ParticleAlignMode2d::Random => "random",
    }
}

fn line_anchor_name(anchor: ParticleLineAnchor2d) -> &'static str {
    match anchor {
        ParticleLineAnchor2d::Center => "center",
        ParticleLineAnchor2d::Start => "start",
        ParticleLineAnchor2d::End => "end",
    }
}

fn light_mode_name(mode: ParticleLightMode2d) -> &'static str {
    match mode {
        ParticleLightMode2d::Source => "source",
        ParticleLightMode2d::Particle => "particle",
    }
}

fn blend_mode_name(blend_mode: ParticleBlendMode2d) -> &'static str {
    match blend_mode {
        ParticleBlendMode2d::Alpha => "alpha",
        ParticleBlendMode2d::Additive => "additive",
        ParticleBlendMode2d::Multiply => "multiply",
        ParticleBlendMode2d::Screen => "screen",
    }
}

fn color_to_hex(color: ColorRgba) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        color_channel(color.r),
        color_channel(color.g),
        color_channel(color.b),
        color_channel(color.a)
    )
}

fn color_channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn fmt_f32(value: f32) -> String {
    if value.abs() < 0.000_001 {
        return "0.0".to_owned();
    }
    let mut formatted = format!("{value:.4}");
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.push('0');
    }
    formatted
}
