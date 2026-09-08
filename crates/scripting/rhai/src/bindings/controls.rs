use super::WorldApi;
use amigo_runtime_control::{ControlValue, RuntimeControlService};
use std::sync::Arc;
#[derive(Clone)]
struct Controls(Option<Arc<RuntimeControlService>>);
type Result<T> = std::result::Result<T, Box<rhai::EvalAltResult>>;
impl Controls {
    fn service(&self) -> Result<&RuntimeControlService> {
        self.0
            .as_deref()
            .ok_or_else(|| "runtime control unavailable".into())
    }
    fn get(&mut self, path: &str) -> Result<rhai::Dynamic> {
        let registry = self.service()?.registry_snapshot();
        let property = registry
            .property(path)
            .ok_or_else(|| Box::<rhai::EvalAltResult>::from(format!("unknown property {path}")))?;
        if !property.readable {
            return Err("property is not readable".into());
        }
        let v = self
            .service()?
            .get(path)
            .map_err(|e| Box::<rhai::EvalAltResult>::from(e.to_string()))?;
        Ok(match v {
            ControlValue::Bool(v) => v.into(),
            ControlValue::I64(v) => v.into(),
            ControlValue::U64(v) => i64::try_from(v)
                .map_err(|_| "integer does not fit Rhai")?
                .into(),
            ControlValue::F64(v) => v.into(),
            ControlValue::String(v) | ControlValue::AssetRef(v) => v.into(),
            ControlValue::Null => ().into(),
            ControlValue::Vec2(v) => v
                .into_iter()
                .map(|v| rhai::Dynamic::from_float(v as f64))
                .collect::<rhai::Array>()
                .into(),
            ControlValue::Vec3(v) => v
                .into_iter()
                .map(|v| rhai::Dynamic::from_float(v as f64))
                .collect::<rhai::Array>()
                .into(),
            ControlValue::Color(v) => v
                .into_iter()
                .map(|v| rhai::Dynamic::from_float(v as f64))
                .collect::<rhai::Array>()
                .into(),
        })
    }
    fn set(&mut self, path: &str, value: rhai::Dynamic) -> Result<()> {
        let value = if value.is_bool() {
            ControlValue::Bool(value.cast())
        } else if value.is_int() {
            ControlValue::I64(value.cast())
        } else if value.is_float() {
            ControlValue::F64(value.cast())
        } else if value.is_string() {
            ControlValue::String(value.into_string().map_err(|_| "invalid string")?)
        } else if value.is_array() {
            let numbers = value
                .cast::<rhai::Array>()
                .into_iter()
                .map(|v| {
                    v.as_float()
                        .or_else(|_| v.as_int().map(|n| n as f64))
                        .map(|v| v as f32)
                })
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|_| "expected numeric vector")?;
            match numbers.as_slice() {
                [x, y] => ControlValue::Vec2([*x, *y]),
                [x, y, z] => ControlValue::Vec3([*x, *y, *z]),
                [r, g, b, a] => ControlValue::Color([*r, *g, *b, *a]),
                _ => return Err("expected Vec2, Vec3 or RGBA".into()),
            }
        } else {
            return Err("unsupported control value".into());
        };
        self.service()?
            .set(path, value)
            .map_err(|e| e.to_string().into())
    }
    fn reset(&mut self, path: &str) -> Result<()> {
        self.service()?
            .reset(path)
            .map_err(|e| e.to_string().into())
    }
}
pub(crate) fn register(engine: &mut rhai::Engine, service: Option<Arc<RuntimeControlService>>) {
    engine
        .register_type_with_name::<Controls>("RuntimeControls")
        .register_get("controls", move |_: &mut WorldApi| {
            Controls(service.clone())
        })
        .register_fn("get", Controls::get)
        .register_fn("set", Controls::set)
        .register_fn("reset", Controls::reset);
}
