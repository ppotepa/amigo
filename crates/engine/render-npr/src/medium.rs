//! Typed, backend-neutral drawing media.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InkMedium {
    pub opacity: f32,
    pub edge_hardness: f32,
}

impl Default for InkMedium {
    fn default() -> Self { Self { opacity: 1.0, edge_hardness: 0.9 } }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphiteMedium {
    pub hardness: f32,
    pub deposit_gain: f32,
    pub tip_radius: f32,
    pub tip_aspect: f32,
    pub paper_coupling: f32,
    pub pressure_exponent: f32,
    pub velocity_exponent: f32,
    pub particle_scale: f32,
    pub filament_count: u8,
    pub filament_spread: f32,
    pub max_optical_density: f32,
}

impl Default for GraphiteMedium {
    fn default() -> Self {
        Self {
            // HB is a good neutral starting point for the PencilAnimation preset.
            hardness: 0.5,
            deposit_gain: 0.7,
            tip_radius: 0.55,
            tip_aspect: 1.15,
            paper_coupling: 0.72,
            pressure_exponent: 1.25,
            velocity_exponent: 0.35,
            particle_scale: 1.0,
            filament_count: 3,
            filament_spread: 0.35,
            max_optical_density: 0.78,
        }
    }
}

impl GraphiteMedium {
    pub fn normalized(self) -> Self {
        Self {
            hardness: self.hardness.clamp(0.0, 1.0),
            deposit_gain: self.deposit_gain.max(0.0),
            tip_radius: self.tip_radius.max(0.01),
            tip_aspect: self.tip_aspect.max(0.1),
            paper_coupling: self.paper_coupling.clamp(0.0, 1.0),
            pressure_exponent: self.pressure_exponent.max(0.01),
            velocity_exponent: self.velocity_exponent.max(0.0),
            particle_scale: self.particle_scale.max(0.01),
            filament_count: self.filament_count.clamp(1, 8),
            filament_spread: self.filament_spread.clamp(0.0, 1.0),
            max_optical_density: self.max_optical_density.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NprMediumDefinition {
    Ink(InkMedium),
    Graphite(GraphiteMedium),
}

impl NprMediumDefinition {
    pub fn kind(self) -> crate::NprMedium {
        match self {
            Self::Ink(_) => crate::NprMedium::Ink,
            Self::Graphite(_) => crate::NprMedium::Graphite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphite_is_normalized_at_the_packet_boundary() {
        let medium = GraphiteMedium { hardness: 2.0, filament_count: 99, ..Default::default() }.normalized();
        assert_eq!(medium.hardness, 1.0);
        assert_eq!(medium.filament_count, 8);
    }
}
