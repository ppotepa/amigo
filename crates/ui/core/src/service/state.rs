#[derive(Debug, Clone, Default, PartialEq)]
pub struct UiStateSnapshot {
    pub text_overrides: BTreeMap<String, String>,
    pub value_overrides: BTreeMap<String, f32>,
    pub curve_overrides: BTreeMap<String, Vec<UiCurvePoint>>,
    pub selected_overrides: BTreeMap<String, String>,
    pub options_overrides: BTreeMap<String, Vec<String>>,
    pub expanded_overrides: BTreeMap<String, bool>,
    pub dropdown_scroll_offsets: BTreeMap<String, f32>,
    pub color_overrides: BTreeMap<String, ColorRgba>,
    pub background_overrides: BTreeMap<String, ColorRgba>,
    pub visibility_overrides: BTreeMap<String, bool>,
    pub enabled_overrides: BTreeMap<String, bool>,
}

#[derive(Debug, Default)]
pub struct UiStateService {
    state: Mutex<UiStateSnapshot>,
}
impl UiStateService {
    pub fn set_text(&self, path: impl Into<String>, value: impl Into<String>) -> bool {
        let path = path.into();
        let value = value.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.text_overrides.get(&path) == Some(&value) {
            return false;
        }
        state.text_overrides.insert(path, value);
        true
    }

    pub fn set_value(&self, path: impl Into<String>, value: f32) -> bool {
        let path = path.into();
        let value = value.clamp(0.0, 1.0);
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.value_overrides.get(&path).copied() == Some(value) {
            return false;
        }
        state.value_overrides.insert(path, value);
        true
    }

    pub fn set_curve_points(&self, path: impl Into<String>, points: Vec<UiCurvePoint>) -> bool {
        let path = path.into();
        let points = normalize_curve_points(&points);
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.curve_overrides.get(&path) == Some(&points) {
            return false;
        }
        state.curve_overrides.insert(path, points);
        true
    }

    pub fn set_selected(&self, path: impl Into<String>, value: impl Into<String>) -> bool {
        let path = path.into();
        let value = value.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.selected_overrides.get(&path) == Some(&value) {
            return false;
        }
        state.selected_overrides.insert(path, value);
        true
    }

    pub fn set_options(&self, path: impl Into<String>, options: Vec<String>) -> bool {
        let path = path.into();
        let options = options
            .into_iter()
            .filter(|option| !option.is_empty())
            .collect::<Vec<_>>();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.options_overrides.get(&path) == Some(&options) {
            return false;
        }
        if let Some(selected) = state.selected_overrides.get(&path) {
            if !options.contains(selected) {
                if let Some(first) = options.first() {
                    state.selected_overrides.insert(path.clone(), first.clone());
                }
            }
        }
        state.options_overrides.insert(path, options);
        true
    }

    pub fn set_curve(&self, path: impl Into<String>, values: Vec<f32>) -> bool {
        self.set_curve_points(path, crate::model::curve_points_from_values(&values))
    }

    pub fn set_expanded(&self, path: impl Into<String>, value: bool) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.expanded_overrides.get(&path).copied() == Some(value) {
            return false;
        }
        state.expanded_overrides.insert(path, value);
        true
    }

    pub fn set_dropdown_scroll_offset(
        &self,
        path: impl Into<String>,
        offset: f32,
        option_count: usize,
        visible_count: usize,
    ) -> bool {
        let path = path.into();
        let max_offset = option_count.saturating_sub(visible_count.max(1)) as f32;
        let offset = if offset.is_finite() {
            offset.clamp(0.0, max_offset)
        } else {
            0.0
        };
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.dropdown_scroll_offsets.get(&path).copied() == Some(offset) {
            return false;
        }
        state.dropdown_scroll_offsets.insert(path, offset);
        true
    }

    pub fn set_color(&self, path: impl Into<String>, color: ColorRgba) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.color_overrides.get(&path).copied() == Some(color) {
            return false;
        }
        state.color_overrides.insert(path, color);
        true
    }

    pub fn set_background(&self, path: impl Into<String>, color: ColorRgba) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.background_overrides.get(&path).copied() == Some(color) {
            return false;
        }
        state.background_overrides.insert(path, color);
        true
    }

    pub fn clear_background(&self, path: &str) -> bool {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .background_overrides
            .remove(path)
            .is_some()
    }

    pub fn show(&self, path: impl Into<String>) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.visibility_overrides.get(&path).copied() == Some(true) {
            return false;
        }
        state.visibility_overrides.insert(path, true);
        true
    }

    pub fn hide(&self, path: impl Into<String>) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.visibility_overrides.get(&path).copied() == Some(false) {
            return false;
        }
        state.visibility_overrides.insert(path, false);
        true
    }

    pub fn enable(&self, path: impl Into<String>) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.enabled_overrides.get(&path).copied() == Some(true) {
            return false;
        }
        state.enabled_overrides.insert(path, true);
        true
    }

    pub fn disable(&self, path: impl Into<String>) -> bool {
        let path = path.into();
        let mut state = self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned");
        if state.enabled_overrides.get(&path).copied() == Some(false) {
            return false;
        }
        state.enabled_overrides.insert(path, false);
        true
    }

    pub fn text_override(&self, path: &str) -> Option<String> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .text_overrides
            .get(path)
            .cloned()
    }

    pub fn value_override(&self, path: &str) -> Option<f32> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .value_overrides
            .get(path)
            .copied()
    }

    pub fn curve_override(&self, path: &str) -> Option<Vec<UiCurvePoint>> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .curve_overrides
            .get(path)
            .cloned()
    }

    pub fn selected_override(&self, path: &str) -> Option<String> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .selected_overrides
            .get(path)
            .cloned()
    }

    pub fn options_override(&self, path: &str) -> Option<Vec<String>> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .options_overrides
            .get(path)
            .cloned()
    }

    pub fn expanded_override(&self, path: &str) -> Option<bool> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .expanded_overrides
            .get(path)
            .copied()
    }

    pub fn dropdown_scroll_offset(&self, path: &str) -> f32 {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .dropdown_scroll_offsets
            .get(path)
            .copied()
            .unwrap_or(0.0)
    }

    pub fn color_override(&self, path: &str) -> Option<ColorRgba> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .color_overrides
            .get(path)
            .copied()
    }

    pub fn background_override(&self, path: &str) -> Option<ColorRgba> {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .background_overrides
            .get(path)
            .copied()
    }

    pub fn is_visible(&self, path: &str) -> bool {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .visibility_overrides
            .get(path)
            .copied()
            .unwrap_or(true)
    }

    pub fn is_enabled(&self, path: &str) -> bool {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .enabled_overrides
            .get(path)
            .copied()
            .unwrap_or(true)
    }

    pub fn snapshot(&self) -> UiStateSnapshot {
        self.state
            .lock()
            .expect("ui state mutex should not be poisoned")
            .clone()
    }

    pub fn clear(&self) {
        *self
            .state
            .lock()
            .expect("ui state mutex should not be poisoned") = UiStateSnapshot::default();
    }
}

