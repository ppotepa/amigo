use crate::{ComponentTypeId, PropertyDescriptor};

#[derive(Debug, Clone)]
pub struct InspectorSchema {
    pub component_type: ComponentTypeId,
    pub title: String,
    pub fields: Vec<PropertyDescriptor>,
}

