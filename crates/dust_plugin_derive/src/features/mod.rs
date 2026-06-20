/// Generates copyWith support.
pub(crate) mod clone_copy_with;
/// Generates `toString` support.
pub(crate) mod debug;
/// Generates equality and hashCode support.
pub(crate) mod eq_hash;
/// Allocates collision-safe generated names.
pub(crate) mod names;
/// Generates validation helpers.
pub(crate) mod validate;
/// Shared generated constructor rendering helpers.
pub(crate) mod writer;

/// Fully qualified `ToString` trait symbol.
pub(crate) const TO_STRING_SYMBOL: &str = "dust_dart::ToString";
/// Fully qualified `Debug` trait symbol.
pub(crate) const DEBUG_SYMBOL: &str = "dust_dart::Debug";
/// Fully qualified `Eq` trait symbol.
pub(crate) const EQ_SYMBOL: &str = "dust_dart::Eq";
/// Fully qualified `CopyWith` trait symbol.
pub(crate) const COPY_WITH_SYMBOL: &str = "dust_dart::CopyWith";
/// Fully qualified `Validate` trait/config symbol.
pub(crate) const VALIDATE_SYMBOL: &str = "dust_dart::Validate";
