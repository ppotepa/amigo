use crate::{ComponentTypeId, PropertyDescriptor};

#[derive(Debug, Clone)]
pub struct InspectorSchema {
    pub component_type: ComponentTypeId,
    pub title: String,
    pub fields: Vec<PropertyDescriptor>,
}

impl InspectorSchema {
    pub fn placeholder(component_type: ComponentTypeId, title: impl Into<String>) -> Self {
        Self {
            component_type,
            title: title.into(),
            fields: Vec::new(),
        }
    }

    pub fn with_field(mut self, field: PropertyDescriptor) -> Self {
        self.fields.push(field);
        self
    }
}

