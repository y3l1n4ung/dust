use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ViewModelFact {
    pub(crate) class_name: String,
    pub(crate) state_type: String,
    pub(crate) args_type: Option<String>,
    pub(crate) initial_source: Option<String>,
    pub(crate) generated_base_class: String,
    pub(crate) import_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ViewModelAnnotation {
    pub(crate) state_type: String,
    pub(crate) args_type: Option<String>,
    pub(crate) initial_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StateFact {
    pub(crate) class_name: String,
    pub(crate) fields: Vec<StateFieldFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StateFieldFact {
    pub(crate) name: String,
    pub(crate) type_source: String,
}
