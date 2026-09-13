use amigo_render_npr::{CoverageMask, NprCoverageSample};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct NprGpuSurface {
    pub position_height: [f32; 4],
    pub normal_tone: [f32; 4],
}
impl From<Option<NprCoverageSample>> for NprGpuSurface {
    fn from(sample: Option<NprCoverageSample>) -> Self {
        sample
            .map(|s| Self {
                position_height: [s.position.x, s.position.y, s.position.z, s.height],
                normal_tone: [s.normal.x, s.normal.y, s.normal.z, s.tone],
            })
            .unwrap_or_default()
    }
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MaskTerm {
    meta: [u32; 4],
    values: [f32; 4],
    direction: [f32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MaskProgram {
    header: [u32; 4],
    terms: [MaskTerm; 128],
}
impl MaskProgram {
    fn compile(mask: &CoverageMask) -> Result<Self, String> {
        mask.validate()?;
        let mut program = Self::zeroed();
        program.append(mask);
        Ok(program)
    }
    fn append(&mut self, mask: &CoverageMask) {
        let mut term = MaskTerm::zeroed();
        match mask {
            CoverageMask::None => return,
            CoverageMask::Multiply { masks } => {
                for mask in masks {
                    self.append(mask);
                }
                return;
            }
            CoverageMask::ToneRange { min, max, invert }
            | CoverageMask::Height { min, max, invert } => {
                term.meta = [
                    if matches!(mask, CoverageMask::Height { .. }) {
                        2
                    } else {
                        1
                    },
                    u32::from(*invert),
                    0,
                    0,
                ];
                term.values = [*min, *max, 0.0, 0.0];
            }
            CoverageMask::NormalDirection {
                direction,
                threshold,
                invert,
            } => {
                term.meta = [3, u32::from(*invert), 0, 0];
                term.values[0] = *threshold;
                let length = direction
                    .iter()
                    .map(|value| value * value)
                    .sum::<f32>()
                    .sqrt();
                term.direction = [
                    direction[0] / length,
                    direction[1] / length,
                    direction[2] / length,
                    0.0,
                ];
            }
            CoverageMask::Noise {
                amount,
                seed,
                invert,
            } => {
                term.meta = [4, u32::from(*invert), *seed as u32, (*seed >> 32) as u32];
                term.values[0] = *amount;
            }
        }
        self.terms[self.header[0] as usize] = term;
        self.header[0] += 1;
    }
}

pub(super) fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("amigo-npr-coverage-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<MaskProgram>() as u64),
            },
            count: None,
        }],
    })
}
impl super::NprPipelines {
    pub(crate) fn mask_binding(
        &self,
        pool: &mut super::NprBufferPool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mask: &CoverageMask,
    ) -> Result<wgpu::BindGroup, String> {
        let program = MaskProgram::compile(mask)?;
        let buffer = pool.upload(
            device,
            queue,
            bytemuck::bytes_of(&program),
            wgpu::BufferUsages::UNIFORM,
            "amigo-npr-coverage-program",
        );
        Ok(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-npr-coverage-binding"),
            layout: &self.mask_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires a graphics adapter; run explicitly for NPR pixel validation"]
    fn gpu_masks_cover_triangle_interiors_and_match_seeded_cpu_noise() {
        use amigo_render_api::{NprBackgroundCommand, NprDrawCommand};
        use amigo_render_npr::*;
        let mut target = crate::backend::WgpuRenderBackend::default()
            .initialize_offscreen(96, 96)
            .expect("NPR GPU test adapter");
        let renderer = super::super::WgpuNprRenderer::new(&target.device, target.format);
        let style = ComicInk {
            tone_mode: NprToneMode::ThreeBand,
            ..ComicInk::default()
        };
        let camera = PerspectiveCamera::cube_default(1.0);
        let mut layers = NprStyleLayers::default();
        for layer in &mut layers.layers {
            layer.enabled = matches!(
                layer.source,
                NprGeometrySource::Paper | NprGeometrySource::FlatFill
            );
        }
        let mut black = style.ink * 0.0;
        black[3] = 1.0;
        layers.layer_mut("fill").unwrap().color_source = NprLayerColorSource::Constant(black);
        let mut packet = build_packet(
            &NprGeometry::canonical_cube(),
            camera,
            [96, 96],
            style,
            7,
            NprDebugView::Final,
        );
        layers.apply_tools(&mut packet, style);
        let mut command = NprDrawCommand::with_preset_and_layers(packet, "mask-test", layers);
        let background = Some(NprBackgroundCommand {
            color: [1.0; 4],
            grain: 0.0,
            tooth: 0.0,
            seed: 0,
        });
        command.layers.layer_mut("fill").unwrap().mask = CoverageMask::Height {
            min: 0.4,
            max: 0.6,
            invert: false,
        };
        renderer
            .render(&mut target, std::slice::from_ref(&command), background)
            .unwrap();
        let pixels = target.read_rgba8_blocking().unwrap();
        let red = |x: usize, y: usize| pixels[(y * 96 + x) * 4];
        assert!(
            red(48, 48) < 10,
            "height band must cover the middle of a face, even though all its corners lie outside the range"
        );
        assert!(
            red(48, 30) > 240,
            "height band must not cover the top of the model"
        );
        for (direction, dark) in [([0.0, 0.0, 1.0], true), ([1.0, 0.0, 0.0], false)] {
            command.layers.layer_mut("fill").unwrap().mask = CoverageMask::NormalDirection {
                direction,
                threshold: 0.8,
                invert: false,
            };
            renderer
                .render(&mut target, std::slice::from_ref(&command), background)
                .unwrap();
            let pixels = target.read_rgba8_blocking().unwrap();
            assert_eq!(pixels[(48 * 96 + 48) * 4] < 10, dark);
        }
        let mask = CoverageMask::Noise {
            amount: 1.0,
            seed: 0x12345678_abcdef01,
            invert: false,
        };
        command.layers.layer_mut("fill").unwrap().mask = mask.clone();
        renderer
            .render(&mut target, std::slice::from_ref(&command), background)
            .unwrap();
        let pixels = target.read_rgba8_blocking().unwrap();
        let position = camera
            .unproject(
                Point2::new(48.5, 48.5),
                camera.normalized_depth(4.0),
                Point2::splat(96.0),
            )
            .unwrap();
        let alpha = mask.evaluate(Some(NprCoverageSample {
            position,
            normal: -camera.forward,
            height: 0.5,
            tone: 0.5,
        }));
        let linear = 1.0 - alpha;
        let srgb = if linear <= 0.0031308 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        };
        assert!(
            (pixels[(48 * 96 + 48) * 4] as f32 - srgb * 255.0).abs() < 3.0,
            "GPU seeded mask must match CPU coverage"
        );
    }
    #[test]
    fn nested_masks_compile_without_losing_seeds_or_inversion() {
        let mask = CoverageMask::Multiply {
            masks: vec![
                CoverageMask::None,
                CoverageMask::Multiply {
                    masks: vec![
                        CoverageMask::Height {
                            min: 0.4,
                            max: 0.6,
                            invert: false,
                        },
                        CoverageMask::Noise {
                            amount: 0.3,
                            seed: 0x12345678_abcdef01,
                            invert: true,
                        },
                    ],
                },
            ],
        };
        let program = MaskProgram::compile(&mask).unwrap();
        assert_eq!(program.header[0], 2);
        assert_eq!(program.terms[0].meta[0], 2);
        assert_eq!(program.terms[1].meta, [4, 1, 0xabcdef01, 0x12345678]);
        assert_eq!(std::mem::size_of::<MaskTerm>(), 48);
        assert_eq!(std::mem::size_of::<MaskProgram>(), 16 + 128 * 48);
    }
}
