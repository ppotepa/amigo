use amigo_fx::{ColorInterpolation, ColorRamp, ColorStop};

use super::{parse_hex_color, ParticlesApi};

impl ParticlesApi {
    #[allow(clippy::too_many_arguments)]
    pub fn set_color_ramp4(
        &mut self,
        entity_name: &str,
        interpolation: &str,
        t0: rhai::FLOAT,
        c0: &str,
        t1: rhai::FLOAT,
        c1: &str,
        t2: rhai::FLOAT,
        c2: &str,
        t3: rhai::FLOAT,
        c3: &str,
    ) -> bool {
        let Some(particles) = self.particles.as_ref() else {
            return false;
        };
        let Some(c0) = parse_hex_color(c0) else {
            return false;
        };
        let Some(c1) = parse_hex_color(c1) else {
            return false;
        };
        let Some(c2) = parse_hex_color(c2) else {
            return false;
        };
        let Some(c3) = parse_hex_color(c3) else {
            return false;
        };
        let interpolation = match interpolation {
            "step" => ColorInterpolation::Step,
            _ => ColorInterpolation::LinearRgb,
        };
        particles.set_color_ramp(
            entity_name,
            ColorRamp {
                interpolation,
                stops: vec![
                    ColorStop {
                        t: t0 as f32,
                        color: c0,
                    },
                    ColorStop {
                        t: t1 as f32,
                        color: c1,
                    },
                    ColorStop {
                        t: t2 as f32,
                        color: c2,
                    },
                    ColorStop {
                        t: t3 as f32,
                        color: c3,
                    },
                ],
            },
        )
    }

    pub fn clear_color_ramp(&mut self, entity_name: &str) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.clear_color_ramp(entity_name))
            .unwrap_or(false)
    }

    pub fn set_curve4(
        &mut self,
        entity_name: &str,
        curve_name: &str,
        v0: rhai::FLOAT,
        v1: rhai::FLOAT,
        v2: rhai::FLOAT,
        v3: rhai::FLOAT,
    ) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_curve4(
                    entity_name,
                    curve_name,
                    v0 as f32,
                    v1 as f32,
                    v2 as f32,
                    v3 as f32,
                )
            })
            .unwrap_or(false)
    }
}
