#[derive(Debug, Default)]
pub struct UiThemeService {
    themes: Mutex<BTreeMap<String, UiTheme>>,
    active_theme_id: Mutex<Option<String>>,
}

impl UiThemeService {
    pub fn register_theme(&self, theme: UiTheme) -> bool {
        let mut themes = self
            .themes
            .lock()
            .expect("ui theme service mutex should not be poisoned");
        let changed = themes.get(&theme.id) != Some(&theme);
        themes.insert(theme.id.clone(), theme);
        changed
    }

    pub fn set_active_theme(&self, theme_id: &str) -> bool {
        if !self
            .themes
            .lock()
            .expect("ui theme service mutex should not be poisoned")
            .contains_key(theme_id)
        {
            return false;
        }
        let mut active = self
            .active_theme_id
            .lock()
            .expect("ui theme active mutex should not be poisoned");
        if active.as_deref() == Some(theme_id) {
            return false;
        }
        *active = Some(theme_id.to_owned());
        true
    }

    pub fn active_theme_id(&self) -> Option<String> {
        self.active_theme_id
            .lock()
            .expect("ui theme active mutex should not be poisoned")
            .clone()
    }

    pub fn active_theme(&self) -> Option<UiTheme> {
        let active = self.active_theme_id()?;
        self.themes
            .lock()
            .expect("ui theme service mutex should not be poisoned")
            .get(&active)
            .cloned()
    }

    pub fn themes(&self) -> Vec<UiTheme> {
        self.themes
            .lock()
            .expect("ui theme service mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        self.themes
            .lock()
            .expect("ui theme service mutex should not be poisoned")
            .clear();
        *self
            .active_theme_id
            .lock()
            .expect("ui theme active mutex should not be poisoned") = None;
    }
}

