//! Integration test harness for Dust driver commands and generated outputs.

#[path = "driver_tests/support.rs"]
mod support;

#[path = "driver_tests/build_outputs.rs"]
mod build_outputs;

#[path = "driver_tests/http_client_outputs.rs"]
mod http_client_outputs;

#[path = "driver_tests/i18n_outputs.rs"]
mod i18n_outputs;

#[path = "driver_tests/i18n_check_outputs.rs"]
mod i18n_check_outputs;

#[path = "driver_tests/i18n_stress_outputs.rs"]
mod i18n_stress_outputs;

#[path = "driver_tests/routing_outputs.rs"]
mod routing_outputs;

#[path = "driver_tests/state_outputs.rs"]
mod state_outputs;

#[path = "driver_tests/db_outputs.rs"]
mod db_outputs;

#[path = "driver_tests/output_policy.rs"]
mod output_policy;

#[path = "driver_tests/build_behavior.rs"]
mod build_behavior;

#[path = "driver_tests/build_validation.rs"]
mod build_validation;

#[path = "driver_tests/commands.rs"]
mod commands;

#[path = "driver_tests/watch.rs"]
mod watch;

#[path = "driver_tests/pub_workspace.rs"]
mod pub_workspace;

#[path = "driver_tests/source_audit.rs"]
mod source_audit;
