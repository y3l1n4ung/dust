#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Dust plugin for typed Flutter Navigator 2.0 route generation."]

/// Route plugin implementation and registration surface.
mod plugin;

pub use plugin::{RoutePlugin, register_plugin};
