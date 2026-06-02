use dust_driver::{BuildRequest, run_build};

use crate::support::{make_workspace, write_file};

#[test]
fn build_rejects_mixin_class_targets_with_clear_diagnostic() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/mixin_target.dart"),
        "part 'mixin_target.g.dart';\n\
         @Derive([ToString(), CopyWith()])\n\
         mixin class MixinTarget {\n\
           final String id;\n\
           const MixinTarget(this.id);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert!(result.has_errors());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("does not support `mixin class` targets like `MixinTarget`")
    }));
    assert!(!workspace.path().join("lib/mixin_target.g.dart").exists());
}
