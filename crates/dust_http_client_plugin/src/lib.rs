#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Dust plugin providing Retrofit-style HTTP client generation for Dart using Dio."]

/// HTTP client plugin implementation and registration surface.
mod plugin;

pub use plugin::{HttpClientPlugin, register_plugin};
