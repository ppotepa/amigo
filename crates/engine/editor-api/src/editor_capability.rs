use crate::InspectorSchema;

pub trait EditorCapability: Send + Sync {
    fn id(&self) -> &'static str;
    fn component_type(&self) -> ComponentTypeId;
    fn inspector_schema(&self) -> InspectorSchema;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentTypeId(pub String);

impl ComponentTypeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

