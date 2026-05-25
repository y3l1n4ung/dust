pub(crate) mod clone_copy_with;
pub(crate) mod debug;
pub(crate) mod eq_hash;
pub(crate) mod writer;

pub(crate) const TO_STRING_SYMBOL: &str = "derive_annotation::ToString";
pub(crate) const DEBUG_SYMBOL: &str = "derive_annotation::Debug";
pub(crate) const EQ_SYMBOL: &str = "derive_annotation::Eq";
pub(crate) const COPY_WITH_SYMBOL: &str = "derive_annotation::CopyWith";
