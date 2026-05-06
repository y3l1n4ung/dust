#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Dust plugin providing Retrofit-style HTTP client generation for Dart using Dio."]

mod plugin;

pub use plugin::{HttpClientPlugin, register_plugin};
