use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use dust_driver::{
    BuildRequest, CheckRequest, CleanRequest, CommandRequest, DoctorRequest, ProgressEvent,
    ProgressPhase, WatchRequest, run, run_build, run_build_with_progress, run_check, run_clean,
    run_doctor, run_watch,
};
use tempfile::tempdir;

fn write_file(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn make_workspace() -> tempfile::TempDir {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");
    root
}

#[test]
fn build_writes_real_outputs_for_multiple_libraries_and_classes() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/models.dart"),
        "part 'models.g.dart';\n\
         @Derive([ToString(), Eq(), CopyWith()])\n\
         class User {\n\
           final String id;\n\
           final int? age;\n\
           const User(this.id, this.age);\n\
         }\n\
         @CopyWith()\n\
         class Team {\n\
           final String name;\n\
           const Team(this.name);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/request.dart"),
        "part 'request.g.dart';\n\
         @CopyWith()\n\
         class Request {\n\
           final String path;\n\
           final Map<String, String> headers;\n\
           const Request.create({required this.path, required this.headers});\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    let models_output = fs::read_to_string(workspace.path().join("lib/models.g.dart")).unwrap();
    let request_output = fs::read_to_string(workspace.path().join("lib/request.g.dart")).unwrap();

    assert!(!result.has_errors());
    assert_eq!(result.build_artifacts.len(), 2);
    assert_eq!(result.cache.as_ref().unwrap().misses, 2);
    assert_eq!(result.cache.as_ref().unwrap().hits, 0);
    assert!(
        result
            .build_artifacts
            .iter()
            .all(|artifact| artifact.written)
    );
    assert!(models_output.contains("part of 'models.dart';"));
    assert!(models_output.contains("mixin _$UserDust {"));
    assert!(models_output.contains("User get _dustSelf => this as User;"));
    assert!(models_output.contains("String toString() {\n    return 'User('"));
    assert!(models_output.contains("'id: ${_dustSelf.id}, '"));
    assert!(models_output.contains("'age: ${_dustSelf.age}'"));
    assert!(models_output.contains("mixin _$TeamDust {"));
    assert!(models_output.contains("Team copyWith({"));
    assert!(models_output.contains("String? name,"));
    assert!(models_output.contains("name ?? _dustSelf.name,"));
    assert!(request_output.contains("part of 'request.dart';"));
    assert!(request_output.contains("mixin _$RequestDust {"));
    assert!(request_output.contains("Request copyWith({"));
    assert!(request_output.contains("String? path,"));
    assert!(request_output.contains("Map<String, String>? headers,"));
    assert!(!request_output.contains("final nextPathSource = path ?? _dustSelf.path;"));
    assert!(!request_output.contains("final nextHeadersSource = headers ?? _dustSelf.headers;"));
    assert!(
        request_output
            .contains("final nextHeaders = Map<String, String>.of(headers ?? _dustSelf.headers);")
    );
    assert!(request_output.contains("return Request.create("));
    assert!(request_output.contains("path: path ?? _dustSelf.path,"));
    assert!(request_output.contains("headers: nextHeaders,"));
}

#[test]
fn build_writes_real_serde_outputs() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/profile.dart"),
        "part 'profile.g.dart';\n\
         @Derive([Serialize(), Deserialize()])\n\
         @SerDe(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)\n\
         class Profile {\n\
           const Profile({required this.id, this.displayName, this.tags = const ['guest']});\n\
           final String id;\n\
           @SerDe(rename: 'display_name', aliases: ['displayName'])\n\
           final String? displayName;\n\
           @SerDe(defaultValue: const ['guest'])\n\
           final List<String> tags;\n\
           factory Profile.fromJson(Map<String, Object?> json) => _$ProfileFromJson(json);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/account.dart"),
        "part 'account.g.dart';\n\
         class Profile {\n\
           const Profile({required this.id});\n\
           final String id;\n\
           factory Profile.fromJson(Map<String, Object?> json) => _$ProfileFromJson(json);\n\
           Map<String, Object?> toJson() => _$ProfileToJson(this);\n\
         }\n\
         @Derive([Serialize(), Deserialize()])\n\
         class Account {\n\
           const Account({required this.profile, required this.metrics, required this.archived});\n\
           final Profile profile;\n\
           final Map<String, List<int>> metrics;\n\
           final bool archived;\n\
           factory Account.fromJson(Map<String, Object?> json) => _$AccountFromJson(json);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    let profile_output = fs::read_to_string(workspace.path().join("lib/profile.g.dart")).unwrap();
    let account_output = fs::read_to_string(workspace.path().join("lib/account.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(
        profile_output.contains("Map<String, Object?> toJson() => _$ProfileToJson(_dustSelf);")
    );
    assert!(profile_output.contains("Profile _$ProfileFromJson(Map<String, Object?> json)"));
    assert!(
        profile_output
            .contains("const allowedKeys = <String>{'id', 'display_name', 'displayName', 'tags'};")
    );
    assert!(profile_output.contains("final tagsValue = json.containsKey('tags') ?"));
    assert!(profile_output.contains(": const ['guest'];"));
    assert!(
        account_output.contains("Map<String, Object?> toJson() => _$AccountToJson(_dustSelf);")
    );
    assert!(account_output.contains("'profile': instance.profile.toJson()"));
    assert!(
        account_output.contains("Profile.fromJson(Map<String, Object?>.from(rawProfile as Map))")
    );
    assert!(account_output.contains("Map<String, Object?>.from(rawMetrics as Map).map((key, value) => MapEntry(key, (value as List<Object?>).map((item) => item as int).toList()))"));
}

#[test]
fn second_build_uses_persistent_cache_under_dot_dart_tool() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );

    let first = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: Some(4),
    });
    let second = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: Some(4),
    });

    assert!(!first.has_errors());
    assert!(!second.has_errors());
    assert!(
        workspace
            .path()
            .join(".dart_tool/dust/build_cache_v1.json")
            .exists()
    );
    assert!(first.build_artifacts[0].written);
    assert!(!second.build_artifacts[0].written);
    assert!(second.build_artifacts[0].cached);
    assert_eq!(second.cache.as_ref().unwrap().hits, 1);
    assert_eq!(second.cache.as_ref().unwrap().misses, 0);
}

#[test]
fn clean_removes_dust_outputs_and_cache_but_keeps_foreign_generated_files() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );

    let built = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });
    assert!(!built.has_errors());

    let foreign_output = workspace.path().join("lib/foreign.g.dart");
    write_file(&foreign_output, "// generated by someone else\n");
    assert!(
        workspace
            .path()
            .join(".dart_tool/dust/build_cache_v1.json")
            .exists()
    );

    let result = run_clean(CleanRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(!result.has_errors());
    let clean = result.clean.as_ref().unwrap();
    assert_eq!(clean.scanned_files, 2);
    assert_eq!(clean.removed_files, 1);
    assert!(clean.cache_cleared);
    assert!(!workspace.path().join("lib/user.g.dart").exists());
    assert!(foreign_output.exists());
    assert!(!workspace.path().join(".dart_tool/dust").exists());
}

#[test]
fn parallel_build_keeps_artifact_order_deterministic() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/z_team.dart"),
        "part 'z_team.g.dart';\n\
         @CopyWith()\n\
         class Team {\n\
           final String id;\n\
           const Team(this.id);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/a_user.dart"),
        "part 'a_user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: Some(4),
    });

    let outputs = result
        .build_artifacts
        .iter()
        .map(|artifact| artifact.output_path.clone())
        .collect::<Vec<_>>();

    assert_eq!(
        outputs,
        vec![
            workspace.path().join("lib/a_user.g.dart"),
            workspace.path().join("lib/z_team.g.dart"),
        ]
    );
}

#[test]
fn build_skips_invalid_library_and_continues_when_fail_fast_is_false() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/bad.dart"),
        "part 'bad.g.dart';\n\
         @CopyWith()\n\
         class Broken {\n\
           final String id;\n\
           final int age;\n\
           const Broken(this.id);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/good.dart"),
        "part 'good.g.dart';\n\
         @CopyWith()\n\
         class Good {\n\
           final String id;\n\
           const Good(this.id);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    assert!(result.has_errors());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains(
            "`CopyWith` requires a constructor that accepts every field on class `Broken`",
        )
    }));
    assert!(!workspace.path().join("lib/bad.g.dart").exists());
    assert!(workspace.path().join("lib/good.g.dart").exists());
}

#[test]
fn build_stops_after_first_error_when_fail_fast_is_true() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/a_bad.dart"),
        "part 'a_bad.g.dart';\n\
         @CopyWith()\n\
         class Broken {\n\
           final String id;\n\
           final int age;\n\
           const Broken(this.id);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/z_good.dart"),
        "part 'z_good.g.dart';\n\
         @CopyWith()\n\
         class Good {\n\
           final String id;\n\
           const Good(this.id);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: true,
        jobs: None,
    });

    assert!(result.has_errors());
    assert_eq!(result.build_artifacts.len(), 1);
    assert!(!workspace.path().join("lib/z_good.g.dart").exists());
}

#[test]
fn build_supports_abstract_and_mixin_clause_shapes_without_unrelated_warnings() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/entity.dart"),
        "part 'entity.g.dart';\n\
         mixin AuditStamp {\n\
           String auditLabel() => 'audited';\n\
         }\n\
         class CatalogNode {\n\
           const CatalogNode();\n\
         }\n\
         @Derive([ToString(), Eq()])\n\
         abstract class Entity extends CatalogNode with AuditStamp {\n\
           final String id;\n\
           const Entity(this.id);\n\
         }\n\
         class EntityView extends Entity {\n\
           const EntityView(super.id);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/tagged_value.dart"),
        "part 'tagged_value.g.dart';\n\
         mixin LabelStamp {\n\
           String labelKind() => 'tagged';\n\
         }\n\
         @Derive([ToString(), Eq(), CopyWith()])\n\
         class TaggedValue with LabelStamp {\n\
           final String code;\n\
           final List<String> aliases;\n\
           const TaggedValue({required this.code, required this.aliases});\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    let entity_output = fs::read_to_string(workspace.path().join("lib/entity.g.dart")).unwrap();
    let tagged_output =
        fs::read_to_string(workspace.path().join("lib/tagged_value.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(
        result.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("could not infer constructor parameter type")),
        "{:?}",
        result.diagnostics
    );
    assert!(entity_output.contains("mixin _$EntityDust {"));
    assert!(entity_output.contains("Entity get _dustSelf => this as Entity;"));
    assert!(entity_output.contains("other is Entity"));
    assert!(tagged_output.contains("mixin _$TaggedValueDust {"));
    assert!(tagged_output.contains("_dustDeepCollectionEquality.equals"));
    assert!(tagged_output.contains("TaggedValue copyWith({"));
    assert!(
        tagged_output
            .contains("final nextAliases = List<String>.of(aliases ?? _dustSelf.aliases);")
    );
}

#[test]
fn build_includes_inherited_fields_for_annotated_subclasses() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/entity.dart"),
        "part 'entity.g.dart';\n\
         @Derive([ToString(), Eq()])\n\
         abstract class Entity with _$EntityDust {\n\
           final String id;\n\
           const Entity(this.id);\n\
         }\n\
         @Derive([ToString(), Eq(), CopyWith()])\n\
         class DetailedEntity extends Entity with _$DetailedEntityDust {\n\
           final String label;\n\
           final List<String> tags;\n\
           const DetailedEntity(super.id, {required this.label, required this.tags});\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    let output = fs::read_to_string(workspace.path().join("lib/entity.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(
        result.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("could not infer constructor parameter type")),
        "{:?}",
        result.diagnostics
    );
    assert!(output.contains("mixin _$DetailedEntityDust {"));
    assert!(output.contains("DetailedEntity get _dustSelf => this as DetailedEntity;"));
    assert!(output.contains("return 'DetailedEntity('"));
    assert!(output.contains("'id: ${_dustSelf.id}, '"));
    assert!(output.contains("'label: ${_dustSelf.label}, '"));
    assert!(output.contains("'tags: ${_dustSelf.tags}'"));
    assert!(output.contains("other.id == _dustSelf.id"));
    assert!(output.contains("DetailedEntity copyWith({"));
    assert!(output.contains("final nextTags = List<String>.of(tags ?? _dustSelf.tags);"));
    assert!(output.contains("return DetailedEntity("));
}

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
    });

    assert!(result.has_errors());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("does not support `mixin class` targets like `MixinTarget`")
    }));
    assert!(!workspace.path().join("lib/mixin_target.g.dart").exists());
}

#[test]
fn check_reports_stale_before_build_and_fresh_after_build() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );

    let first_check = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });
    let build = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });
    let second_check = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    assert_eq!(first_check.checked_libraries.len(), 1);
    assert!(first_check.checked_libraries[0].stale);
    assert!(build.build_artifacts[0].written);
    assert_eq!(second_check.checked_libraries.len(), 1);
    assert!(!second_check.checked_libraries[0].stale);
}

#[test]
fn doctor_reports_workspace_and_registered_plugins() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );

    let result = run_doctor(DoctorRequest {
        cwd: workspace.path().to_path_buf(),
    });
    let doctor = result.doctor.unwrap();

    assert_eq!(doctor.root, workspace.path());
    assert_eq!(doctor.library_count, 1);
    assert_eq!(
        doctor.plugin_names,
        vec![
            "dust_plugin_derive".to_owned(),
            "dust_plugin_serde".to_owned()
        ]
    );
    assert_eq!(
        doctor.libraries,
        vec![workspace.path().join("lib/user.dart")]
    );
}

#[test]
fn run_dispatches_supported_commands() {
    let workspace = make_workspace();
    let result = run(CommandRequest::Doctor(DoctorRequest {
        cwd: workspace.path().to_path_buf(),
    }));

    assert!(result.doctor.is_some());
}

#[test]
fn build_emits_progress_events() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&events);

    let result = run_build_with_progress(
        BuildRequest {
            cwd: workspace.path().to_path_buf(),
            fail_fast: false,
            jobs: Some(2),
        },
        move |event| {
            sink.lock().unwrap().push(event);
        },
    );

    assert!(!result.has_errors());
    let events = events.lock().unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        ProgressEvent::StartedBatch {
            phase: ProgressPhase::Build,
            total: 1
        }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        ProgressEvent::FinishedLibrary {
            phase: ProgressPhase::Build,
            completed: 1,
            total: 1,
            cached: false,
            ..
        }
    )));
}

#[test]
fn watch_runs_initial_build_for_existing_candidates() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );

    let result = run_watch(WatchRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        poll_interval_ms: 10,
        max_cycles: Some(1),
    });

    assert!(workspace.path().join("lib/user.g.dart").exists());
    assert_eq!(result.build_artifacts.len(), 1);
    assert_eq!(result.watch.as_ref().unwrap().cycles, 1);
    assert_eq!(result.watch.as_ref().unwrap().rebuild_batches, 0);
}

#[test]
fn watch_rebuilds_only_the_changed_library() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/team.dart"),
        "part 'team.g.dart';\n\
         @CopyWith()\n\
         class Team {\n\
           final String name;\n\
           const Team(this.name);\n\
         }\n",
    );

    let root = workspace.path().to_path_buf();
    let user_path = root.join("lib/user.dart");
    let modifier = thread::spawn(move || {
        thread::sleep(Duration::from_millis(25));
        write_file(
            &user_path,
            "part 'user.g.dart';\n\
             @ToString()\n\
             class User {\n\
               final String id;\n\
               final int age;\n\
               const User(this.id, this.age);\n\
             }\n",
        );
    });

    let result = run_watch(WatchRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        poll_interval_ms: 20,
        max_cycles: Some(3),
    });
    modifier.join().unwrap();

    let watch = result.watch.unwrap();
    let user_output = fs::read_to_string(workspace.path().join("lib/user.g.dart")).unwrap();
    let team_output = fs::read_to_string(workspace.path().join("lib/team.g.dart")).unwrap();

    assert_eq!(watch.rebuild_batches, 1);
    assert_eq!(
        watch.rebuilt_libraries,
        vec![workspace.path().join("lib/user.dart")]
    );
    assert!(user_output.contains("return 'User('"));
    assert!(user_output.contains("'id: ${_dustSelf.id}, '"));
    assert!(user_output.contains("'age: ${_dustSelf.age}'"));
    assert!(team_output.contains("Team copyWith({"));
    assert_eq!(result.build_artifacts.len(), 3);
}

#[test]
fn watch_rebuilds_all_libraries_when_package_config_changes() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/team.dart"),
        "part 'team.g.dart';\n\
         @CopyWith()\n\
         class Team {\n\
           final String name;\n\
           const Team(this.name);\n\
         }\n",
    );

    let root = workspace.path().to_path_buf();
    let package_config = root.join(".dart_tool/package_config.json");
    let modifier = thread::spawn(move || {
        thread::sleep(Duration::from_millis(25));
        write_file(&package_config, "{\"configVersion\":2}\n");
    });

    let result = run_watch(WatchRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        poll_interval_ms: 20,
        max_cycles: Some(3),
    });
    modifier.join().unwrap();

    let watch = result.watch.unwrap();
    assert_eq!(watch.rebuild_batches, 1);
    assert_eq!(
        watch.rebuilt_libraries,
        vec![
            workspace.path().join("lib/team.dart"),
            workspace.path().join("lib/user.dart"),
        ]
    );
}

#[test]
fn run_dispatches_watch_requests() {
    let workspace = make_workspace();
    let result = run(CommandRequest::Watch(WatchRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        poll_interval_ms: 10,
        max_cycles: Some(1),
    }));

    assert!(result.watch.is_some());
}
