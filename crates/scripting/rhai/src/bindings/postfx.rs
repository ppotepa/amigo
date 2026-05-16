use std::sync::Arc;

use amigo_2d_post_fx::{
    ColorQuantize2d, ColorRamp2d, PostFx2d, PostFx2dService, RainGlass2d, RainGlassPatch,
};

#[derive(Clone)]
pub struct PostFxApi {
    pub(crate) post_fx: Option<Arc<PostFx2dService>>,
}

impl PostFxApi {
    pub fn count(&mut self) -> rhai::INT {
        self.post_fx
            .as_ref()
            .map(|service| service.frame_effect_count() as rhai::INT)
            .unwrap_or(0)
    }

    pub fn list(&mut self) -> rhai::Array {
        self.post_fx
            .as_ref()
            .map(|service| {
                service
                    .frame_effects()
                    .into_iter()
                    .enumerate()
                    .map(|(index, effect)| item_map(index, effect))
                    .collect::<rhai::Array>()
            })
            .unwrap_or_default()
    }

    pub fn item(&mut self, index: rhai::INT) -> PostFxItemRef {
        PostFxItemRef {
            post_fx: self.post_fx.clone(),
            index: index.max(0) as usize,
        }
    }

    pub fn frame_effect_enabled(&mut self, index: rhai::INT) -> bool {
        self.post_fx
            .as_ref()
            .map(|service| service.frame_effect_enabled(index.max(0) as usize))
            .unwrap_or(false)
    }

    pub fn set_frame_effect_enabled(&mut self, index: rhai::INT, enabled: bool) -> bool {
        self.post_fx
            .as_ref()
            .map(|service| service.set_frame_effect_enabled(index.max(0) as usize, enabled))
            .unwrap_or(false)
    }

    pub fn color_quantize_palette_size(&mut self) -> rhai::INT {
        self.post_fx
            .as_ref()
            .and_then(|service| {
                service
                    .frame_effects()
                    .into_iter()
                    .find_map(|effect| match effect {
                        PostFx2d::ColorQuantize(effect) => Some(effect.palette_size as rhai::INT),
                        _ => None,
                    })
            })
            .unwrap_or(0)
    }

    pub fn set_color_quantize_palette_size(&mut self, value: rhai::INT) -> rhai::INT {
        self.update_color_quantize(|effect| {
            effect.palette_size = value.clamp(2, 256) as u32;
        })
    }

    pub fn adjust_color_quantize_palette_size(&mut self, delta: rhai::INT) -> rhai::INT {
        self.update_color_quantize(|effect| {
            let next = effect.palette_size as rhai::INT + delta;
            effect.palette_size = next.clamp(2, 256) as u32;
        })
    }

    pub fn set_color_quantize(&mut self, updates: &str) -> bool {
        let mut applied = false;
        self.update_color_quantize(|effect| {
            for token in updates.split_whitespace() {
                let Some((field, value)) = token.split_once('=') else {
                    continue;
                };
                applied |= set_color_quantize_field(effect, field, value);
            }
        });
        applied
    }

    pub fn color_ramp_palette_size(&mut self) -> rhai::INT {
        self.post_fx
            .as_ref()
            .and_then(|service| {
                service
                    .frame_effects()
                    .into_iter()
                    .find_map(|effect| match effect {
                        PostFx2d::ColorRamp(effect) => Some(effect.palette_size as rhai::INT),
                        _ => None,
                    })
            })
            .unwrap_or(0)
    }

    pub fn set_color_ramp(&mut self, updates: &str) -> bool {
        let mut applied = false;
        self.update_color_ramp(|effect| {
            for token in updates.split_whitespace() {
                let Some((field, value)) = token.split_once('=') else {
                    continue;
                };
                applied |= set_color_ramp_field(effect, field, value);
            }
        });
        applied
    }

    pub fn adjust_color_ramp_palette_size(&mut self, delta: rhai::INT) -> rhai::INT {
        self.update_color_ramp(|effect| {
            let next = effect.palette_size as rhai::INT + delta;
            effect.palette_size = next.clamp(2, 256) as u32;
        })
    }

    pub fn set_rain_glass(&mut self, updates: &str) -> bool {
        let mut applied = false;

        let changed = self.update_rain_glass(|rain| {
            applied = RainGlassPatch::apply_update_string(rain, updates);
        });

        changed && applied
    }

    pub fn set_rain_glass_bool(&mut self, field: &str, value: bool) -> bool {
        let mut applied = false;

        let changed = self.update_rain_glass(|rain| {
            applied = RainGlassPatch::apply_bool(rain, field, value);
        });

        changed && applied
    }

    pub fn set_rain_glass_int(&mut self, field: &str, value: rhai::INT) -> bool {
        let mut applied = false;

        let changed = self.update_rain_glass(|rain| {
            applied = RainGlassPatch::apply_int(rain, field, value as i64);
        });

        changed && applied
    }

    pub fn set_rain_glass_float(&mut self, field: &str, value: rhai::FLOAT) -> bool {
        let mut applied = false;

        let changed = self.update_rain_glass(|rain| {
            applied = RainGlassPatch::apply_float(rain, field, value as f32);
        });

        changed && applied
    }

    pub fn set_rain_glass_debug(&mut self, value: &str) -> bool {
        let mut applied = false;

        let changed = self.update_rain_glass(|rain| {
            applied = RainGlassPatch::apply_debug(rain, value);
        });

        changed && applied
    }

    pub fn set_rain_glass_compose(&mut self, value: &str) -> bool {
        let mut applied = false;

        let changed = self.update_rain_glass(|rain| {
            applied = RainGlassPatch::apply_compose(rain, value);
        });

        changed && applied
    }

    pub fn apply_rain_glass_preset(&mut self, value: &str) -> bool {
        let mut applied = false;

        let changed = self.update_rain_glass(|rain| {
            applied = RainGlassPatch::apply_preset(rain, value);
        });

        changed && applied
    }

    fn update_rain_glass(&self, update: impl FnOnce(&mut RainGlass2d)) -> bool {
        let Some(service) = self.post_fx.as_ref() else {
            return false;
        };

        let mut stack = service.frame_stack().unwrap_or_default();
        let index = stack
            .effects
            .iter()
            .position(|effect| matches!(effect, PostFx2d::RainGlass(_)))
            .unwrap_or_else(|| {
                stack
                    .effects
                    .push(PostFx2d::RainGlass(RainGlass2d::default()));
                stack.effects.len() - 1
            });

        let mut rain = match stack.effects[index] {
            PostFx2d::RainGlass(rain) => rain,
            _ => RainGlass2d::default(),
        };
        update(&mut rain);
        stack.effects[index] = PostFx2d::RainGlass(rain.normalized());
        service.set_scoped_stacks(vec![
            amigo_2d_post_fx::ScopedPostFx2dStack::from_frame_stack(stack.normalized()),
        ]);
        true
    }

    fn update_color_quantize(&self, update: impl FnOnce(&mut ColorQuantize2d)) -> rhai::INT {
        let Some(service) = self.post_fx.as_ref() else {
            return 0;
        };

        let mut stack = service.frame_stack().unwrap_or_default();
        let index = stack
            .effects
            .iter()
            .position(|effect| matches!(effect, PostFx2d::ColorQuantize(_)))
            .unwrap_or_else(|| {
                stack
                    .effects
                    .push(PostFx2d::ColorQuantize(ColorQuantize2d::default()));
                stack.effects.len() - 1
            });

        let mut effect = match stack.effects[index] {
            PostFx2d::ColorQuantize(effect) => effect,
            _ => ColorQuantize2d::default(),
        };
        update(&mut effect);
        let effect = effect.normalized();
        let palette_size = effect.palette_size as rhai::INT;
        stack.effects[index] = PostFx2d::ColorQuantize(effect);
        service.set_scoped_stacks(vec![
            amigo_2d_post_fx::ScopedPostFx2dStack::from_frame_stack(stack.normalized()),
        ]);
        palette_size
    }

    fn update_color_ramp(&self, update: impl FnOnce(&mut ColorRamp2d)) -> rhai::INT {
        let Some(service) = self.post_fx.as_ref() else {
            return 0;
        };

        let mut stack = service.frame_stack().unwrap_or_default();
        let index = stack
            .effects
            .iter()
            .position(|effect| matches!(effect, PostFx2d::ColorRamp(_)))
            .unwrap_or_else(|| {
                stack
                    .effects
                    .push(PostFx2d::ColorRamp(ColorRamp2d::default()));
                stack.effects.len() - 1
            });

        let mut effect = match stack.effects[index] {
            PostFx2d::ColorRamp(effect) => effect,
            _ => ColorRamp2d::default(),
        };
        update(&mut effect);
        let effect = effect.normalized();
        let palette_size = effect.palette_size as rhai::INT;
        stack.effects[index] = PostFx2d::ColorRamp(effect);
        service.set_scoped_stacks(vec![
            amigo_2d_post_fx::ScopedPostFx2dStack::from_frame_stack(stack.normalized()),
        ]);
        palette_size
    }
}

#[derive(Clone)]
pub struct PostFxItemRef {
    post_fx: Option<Arc<PostFx2dService>>,
    index: usize,
}

impl PostFxItemRef {
    pub fn exists(&mut self) -> bool {
        self.effect().is_some()
    }

    pub fn index(&mut self) -> rhai::INT {
        self.index as rhai::INT
    }

    pub fn name(&mut self) -> String {
        self.effect()
            .map(|effect| effect.kind().to_owned())
            .unwrap_or_default()
    }

    pub fn active(&mut self) -> bool {
        self.effect()
            .map(|effect| effect.is_active())
            .unwrap_or(false)
    }

    pub fn inspect_index(&self) -> usize {
        self.index
    }

    pub fn inspect_label(&self) -> Option<String> {
        self.effect()
            .map(|effect| format!("{} #{}", effect.kind(), self.index))
    }

    fn effect(&self) -> Option<PostFx2d> {
        self.post_fx
            .as_ref()
            .and_then(|service| service.frame_effect_raw(self.index))
    }
}

fn item_map(index: usize, effect: PostFx2d) -> rhai::Dynamic {
    let mut map = rhai::Map::new();
    map.insert("index".into(), (index as rhai::INT).into());
    map.insert("name".into(), effect.clone().kind().to_owned().into());
    map.insert("active".into(), effect.is_active().into());
    map.into()
}

fn set_color_quantize_field(effect: &mut ColorQuantize2d, field: &str, value: &str) -> bool {
    let field = field.trim();
    match field {
        "palette" | "palette_size" | "colors" => value
            .parse::<u32>()
            .map(|value| {
                effect.palette_size = value.clamp(2, 256);
            })
            .is_ok(),
        "seed" => value
            .parse::<u32>()
            .map(|value| {
                effect.seed = value;
            })
            .is_ok(),
        "dither" | "dither_strength" => set_color_quantize_f32(value, |value| {
            effect.dither_strength = value;
        }),
        "scale" | "dither_scale" => set_color_quantize_f32(value, |value| {
            effect.dither_scale = value;
        }),
        "layered" | "layered_dither" => set_color_quantize_f32(value, |value| {
            effect.layered_dither = value;
        }),
        "opacity" => set_color_quantize_f32(value, |value| {
            effect.opacity = value;
        }),
        "luma" | "luma_preserve" => set_color_quantize_f32(value, |value| {
            effect.luma_preserve = value;
        }),
        "highlight" | "highlight_bias" | "light_bias" => set_color_quantize_f32(value, |value| {
            effect.highlight_bias = value;
        }),
        "shadow" | "shadow_bias" => set_color_quantize_f32(value, |value| {
            effect.shadow_bias = value;
        }),
        "contrast" => set_color_quantize_f32(value, |value| {
            effect.contrast = value;
        }),
        "saturation" | "sat" => set_color_quantize_f32(value, |value| {
            effect.saturation = value;
        }),
        "gamma" => set_color_quantize_f32(value, |value| {
            effect.gamma = value;
        }),
        _ => false,
    }
}

fn set_color_quantize_f32(value: &str, set: impl FnOnce(f32)) -> bool {
    value
        .parse::<f32>()
        .map(|value| {
            set(value);
        })
        .is_ok()
}

fn set_color_ramp_field(effect: &mut ColorRamp2d, field: &str, value: &str) -> bool {
    let field = field.trim();
    match field {
        "palette" | "palette_size" | "colors" => value
            .parse::<u32>()
            .map(|value| {
                effect.palette_size = value.clamp(2, 256);
            })
            .is_ok(),
        "seed" => value
            .parse::<u32>()
            .map(|value| {
                effect.seed = value;
            })
            .is_ok(),
        "dither" | "dither_strength" => set_color_quantize_f32(value, |value| {
            effect.dither_strength = value;
        }),
        "scale" | "dither_scale" => set_color_quantize_f32(value, |value| {
            effect.dither_scale = value;
        }),
        "layered" | "layered_dither" => set_color_quantize_f32(value, |value| {
            effect.layered_dither = value;
        }),
        "opacity" => set_color_quantize_f32(value, |value| {
            effect.opacity = value;
        }),
        "luma" | "luma_preserve" => set_color_quantize_f32(value, |value| {
            effect.luma_preserve = value;
        }),
        "highlight" | "highlight_bias" | "light_bias" => set_color_quantize_f32(value, |value| {
            effect.highlight_bias = value;
        }),
        "shadow" | "shadow_bias" => set_color_quantize_f32(value, |value| {
            effect.shadow_bias = value;
        }),
        "contrast" => set_color_quantize_f32(value, |value| {
            effect.contrast = value;
        }),
        "saturation" | "sat" => set_color_quantize_f32(value, |value| {
            effect.saturation = value;
        }),
        "gamma" => set_color_quantize_f32(value, |value| {
            effect.gamma = value;
        }),
        _ => false,
    }
}
