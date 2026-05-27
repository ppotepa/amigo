use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct RenderContributionsDocument {
    pub roles: BTreeMap<String, bool>,
}

impl RenderContributionsDocument {
    pub fn is_empty(&self) -> bool {
        self.roles.is_empty()
    }

    pub fn get(&self, role: &str) -> Option<bool> {
        self.roles.get(role).copied()
    }

    pub fn set(&mut self, role: impl Into<String>, enabled: bool) {
        self.roles.insert(role.into(), enabled);
    }

    pub fn with_defaults(
        mut self,
        defaults: impl IntoIterator<Item = (&'static str, bool)>,
    ) -> Self {
        for (role, enabled) in defaults {
            self.roles.entry(role.to_owned()).or_insert(enabled);
        }
        self
    }

    pub fn into_roles(self) -> BTreeMap<String, bool> {
        self.roles
    }
}
