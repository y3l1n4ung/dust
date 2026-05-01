use dust_diagnostics::Diagnostic;
use dust_workspace::SourceLibrary;

use crate::build::{
    batch::BatchConfig,
    process::LoadedLibraryInput,
    support::{CacheFingerprint, load_library_input},
    work::{available_worker_count, round_robin_groups},
};

pub(super) fn load_library_inputs(
    config: BatchConfig<'_>,
    libraries: &[SourceLibrary],
) -> Vec<Result<LoadedLibraryInput, Diagnostic>> {
    let threads = available_worker_count(libraries.len(), None);
    let groups = round_robin_groups(libraries.iter().enumerate(), threads);

    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(groups.len());
        for group in groups {
            handles.push(scope.spawn(move || {
                group
                    .into_iter()
                    .map(|(index, library)| {
                        let cache_fingerprint = config
                            .cache
                            .get(config.cache_root, &library.source_path)
                            .map(|entry| CacheFingerprint {
                                source_hash: entry.source_hash,
                                package_config_hash: entry.package_config_hash,
                                tool_hash: entry.tool_hash,
                            });
                        let input = load_library_input(
                            library,
                            cache_fingerprint,
                            config.package_config_hash,
                            config.tool_hash,
                        );
                        (index, input)
                    })
                    .collect::<Vec<_>>()
            }));
        }

        let mut ordered = (0..libraries.len()).map(|_| None).collect::<Vec<_>>();
        for handle in handles {
            for (index, input) in handle.join().expect("load thread must not panic") {
                ordered[index] = Some(input);
            }
        }

        ordered
            .into_iter()
            .map(|entry| entry.expect("every library load slot must be filled"))
            .collect()
    })
}
