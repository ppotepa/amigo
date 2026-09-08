//! Typed tool response curves used by the domain tessellator.
//! A tool is deliberately separate from the hand gesture: a confident pencil
//! stroke and a hesitant pen stroke are both valid combinations.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StrokeTool {
    Pencil,
    Fineliner,
    Nib,
    Brush,
}

impl Default for StrokeTool {
    fn default() -> Self {
        Self::Fineliner
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolResponse {
    pub width_scale: f32,
    pub pressure_width: f32,
    pub pressure_alpha: f32,
    pub grain: f32,
    pub edge_softness: f32,
}

impl StrokeTool {
    pub fn response(self, pressure: f32, tangent_angle: f32, hardness: f32) -> ToolResponse {
        let pressure = pressure.clamp(0.0, 1.0);
        let hardness = hardness.clamp(0.0, 1.0);
        match self {
            Self::Pencil => ToolResponse {
                width_scale: 0.92 + pressure * 0.18,
                pressure_width: 0.55 + pressure * 0.75,
                pressure_alpha: 0.30 + pressure * 0.70,
                grain: 0.28 + (1.0 - hardness) * 0.55,
                edge_softness: 0.20 + (1.0 - hardness) * 0.45,
            },
            Self::Fineliner => ToolResponse {
                width_scale: 1.0,
                pressure_width: 1.0,
                // A fineliner is the compatibility baseline: pressure can
                // still participate in custom profiles, but the default ink
                // remains fully opaque like the original Comic Ink path.
                pressure_alpha: 1.0,
                grain: 0.04 + (1.0 - hardness) * 0.12,
                edge_softness: 0.04,
            },
            Self::Nib => {
                // A broad nib exposes more width when its direction crosses the
                // nib's long axis. The angle is supplied by the gesture sample.
                let direction = tangent_angle.cos().abs();
                ToolResponse {
                    width_scale: 0.78 + direction * 0.62,
                    pressure_width: 0.72 + pressure * 0.52,
                    pressure_alpha: 0.76 + pressure * 0.24,
                    grain: 0.06 + (1.0 - hardness) * 0.18,
                    edge_softness: 0.06,
                }
            }
            Self::Brush => ToolResponse {
                width_scale: 0.72 + pressure * 0.90,
                pressure_width: 0.48 + pressure * 0.95,
                pressure_alpha: 0.24 + pressure * 0.76,
                grain: 0.10 + (1.0 - hardness) * 0.34,
                edge_softness: 0.14 + (1.0 - hardness) * 0.35,
            },
        }
    }
}
