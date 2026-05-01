#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StateScope {
    Scene,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateKey {
    scope: StateScope,
    name: String,
}

impl StateKey {
    pub fn scene(name: impl Into<String>) -> Self {
        Self {
            scope: StateScope::Scene,
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneStateValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Default)]
pub struct SceneStateService {
    values: Mutex<BTreeMap<StateKey, SceneStateValue>>,
}

#[derive(Debug, Default)]
pub struct SessionStateService {
    values: Mutex<BTreeMap<String, SceneStateValue>>,
}

