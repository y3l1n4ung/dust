use dust_ir::{ConfigApplicationIr, NormalizedConfigIr, StateConfigIr, StateModeIr};

/// Normalizes class-level `ViewModel` configuration after symbol resolution.
pub(crate) fn normalize_state(configs: &mut [ConfigApplicationIr]) {
    let Some(index) = configs
        .iter()
        .position(|config| config.symbol.0 == "dust_flutter::ViewModel")
    else {
        return;
    };
    let Some(state) = state_config(&configs[index]) else {
        return;
    };
    configs[index].normalized = Some(NormalizedConfigIr::State(state));
}

/// Builds typed view model configuration from one resolved application.
fn state_config(config: &ConfigApplicationIr) -> Option<StateConfigIr> {
    let state_type = config
        .named_type("state")
        .or_else(|| config.positional_type(0))?;
    let args_type = config.named_type("args");
    let initial_source = config.named_expression_source("initial");
    let mode_source = config.named_expression_source("mode");
    let mode = match mode_source.as_deref().map(str::trim) {
        Some(source) if source == "async" || source.ends_with(".async") => StateModeIr::Async,
        _ => StateModeIr::Sync,
    };
    Some(StateConfigIr {
        state_type,
        args_type,
        initial_source,
        mode_source,
        mode,
    })
}
