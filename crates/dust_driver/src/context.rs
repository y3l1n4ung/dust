use std::path::Path;

use dust_cache::WorkspaceCache;
use dust_diagnostics::Diagnostic;
use dust_plugin_api::PluginRegistry;
use dust_resolver::SymbolCatalog;
use dust_workspace::{WorkspacePlan, discover_workspace};

use crate::{
    build::{codegen_tool_hash, default_registry, read_package_config_hash},
    catalog::build_symbol_catalog,
    result::CacheReport,
};

pub(crate) struct DriverContext {
    pub(crate) workspace: WorkspacePlan,
    pub(crate) registry: PluginRegistry,
    pub(crate) catalog: SymbolCatalog,
}

impl DriverContext {
    pub(crate) fn load(cwd: &Path) -> Result<Self, Diagnostic> {
        let workspace = discover_workspace(cwd)?;
        let registry = default_registry();
        let catalog = build_symbol_catalog(&registry);

        Ok(Self {
            workspace,
            registry,
            catalog,
        })
    }
}

pub(crate) struct CachedDriverContext {
    pub(crate) workspace: WorkspacePlan,
    pub(crate) registry: PluginRegistry,
    pub(crate) catalog: SymbolCatalog,
    pub(crate) tool_hash: u64,
    pub(crate) package_config_hash: u64,
    pub(crate) cache: WorkspaceCache,
    pub(crate) cache_report: CacheReport,
}

impl CachedDriverContext {
    pub(crate) fn load(cwd: &Path) -> Result<Self, Diagnostic> {
        let DriverContext {
            workspace,
            registry,
            catalog,
        } = DriverContext::load(cwd)?;
        let tool_hash = codegen_tool_hash();
        let package_config_hash = read_package_config_hash(&workspace.package_config.path)?;
        let cache = load_workspace_cache(&workspace)?;
        let cache_report = CacheReport {
            path: cache.path().to_path_buf(),
            ..CacheReport::default()
        };

        Ok(Self {
            workspace,
            registry,
            catalog,
            tool_hash,
            package_config_hash,
            cache,
            cache_report,
        })
    }
}

fn load_workspace_cache(workspace: &WorkspacePlan) -> Result<WorkspaceCache, Diagnostic> {
    WorkspaceCache::load(&workspace.cache_root).map_err(|error| {
        Diagnostic::error(format!(
            "failed to load Dust cache `{}`: {error}",
            workspace
                .cache_root
                .join(".dart_tool/dust/build_cache_v1.json")
                .display()
        ))
    })
}
