use std::path::Path;

use dust_cache::WorkspaceCache;
use dust_diagnostics::Diagnostic;
use dust_plugin_api::PluginRegistry;
use dust_resolver::SymbolCatalog;
use dust_workspace::{SupportedAnnotations, WorkspacePlan, discover_workspace};

use crate::{
    build::{
        CodegenToolHash, RegistrySelection, codegen_tool_hash_for_selection,
        read_workspace_config_hash, registry_for_selection,
    },
    catalog::build_symbol_catalog,
    result::CacheReport,
};

/// Workspace context shared by commands that do not need cache metadata.
pub(crate) struct DriverContext {
    /// Discovered workspace plan.
    pub(crate) workspace: WorkspacePlan,
    /// Plugin registry selected for the command.
    pub(crate) registry: PluginRegistry,
    /// Resolver catalog derived from the registry.
    pub(crate) catalog: SymbolCatalog,
}

impl DriverContext {
    /// Discovers the workspace and builds the selected registry and catalog.
    pub(crate) fn load(cwd: &Path, selection: RegistrySelection) -> Result<Self, Diagnostic> {
        let registry = registry_for_selection(selection);
        let supported_annotations: SupportedAnnotations =
            registry.all_supported_annotations().into_iter().collect();
        let workspace = discover_workspace(cwd, &supported_annotations)?;
        let catalog = build_symbol_catalog(&registry);

        Ok(Self {
            workspace,
            registry,
            catalog,
        })
    }
}

/// Workspace context shared by commands that need cache metadata.
pub(crate) struct CachedDriverContext {
    /// Discovered workspace plan.
    pub(crate) workspace: WorkspacePlan,
    /// Plugin registry selected for the command.
    pub(crate) registry: PluginRegistry,
    /// Resolver catalog derived from the registry.
    pub(crate) catalog: SymbolCatalog,
    /// Hash of active code generation logic and plugins.
    pub(crate) tool_hash: CodegenToolHash,
    /// Hash of package and Dust configuration files.
    pub(crate) package_config_hash: u64,
    /// Mutable workspace cache.
    pub(crate) cache: WorkspaceCache,
    /// Cache report initialized with the cache storage path.
    pub(crate) cache_report: CacheReport,
}

impl CachedDriverContext {
    /// Loads workspace, registry, catalog, hashes, and cache state.
    pub(crate) fn load(cwd: &Path, selection: RegistrySelection) -> Result<Self, Diagnostic> {
        let DriverContext {
            workspace,
            registry,
            catalog,
        } = DriverContext::load(cwd, selection)?;
        let tool_hash = codegen_tool_hash_for_selection(selection);
        let package_config_hash = read_workspace_config_hash(
            &workspace.package_config.path,
            workspace.dust_config.path.as_deref(),
        )?;
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

/// Loads the workspace cache and converts IO failures into diagnostics.
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
