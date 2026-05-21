#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "State management generation plugin for Dust."]

mod plugin;

pub use plugin::{StatePlugin, register_plugin};
