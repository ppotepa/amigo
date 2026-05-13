use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct ScriptRandomState {
    seed: Mutex<u64>,
}

impl Default for ScriptRandomState {
    fn default() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0xA11C_EED5_5EED);
        Self {
            seed: Mutex::new(seed | 1),
        }
    }
}

impl ScriptRandomState {
    fn next_unit(&self) -> f32 {
        let mut seed = self
            .seed
            .lock()
            .expect("script random mutex should not be poisoned");
        let mut value = *seed;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        *seed = value;
        (value as f64 / u64::MAX as f64) as f32
    }
}

#[derive(Clone)]
pub struct RandomApi {
    pub(crate) state: Arc<ScriptRandomState>,
}

impl RandomApi {
    pub fn range(&mut self, min: rhai::FLOAT, max: rhai::FLOAT) -> rhai::FLOAT {
        if !min.is_finite() || !max.is_finite() {
            return min;
        }
        if (max - min).abs() <= rhai::FLOAT::EPSILON {
            return min;
        }
        let low = min.min(max);
        let high = min.max(max);
        low + (high - low) * self.state.next_unit() as rhai::FLOAT
    }

    pub fn chance(&mut self, probability: rhai::FLOAT) -> bool {
        if !probability.is_finite() {
            return false;
        }
        self.state.next_unit() <= probability.clamp(0.0, 1.0) as f32
    }
}

