#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "State management generation plugin for Dust."]

/// State plugin implementation modules.
mod plugin;

pub use plugin::{StatePlugin, register_plugin};
