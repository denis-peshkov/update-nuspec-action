#[path = "../support/mod.rs"]
mod support;

use std::fs;
use std::path::PathBuf;

use update_nuspec::cli::args::CliRunOptions;
use update_nuspec::cli::run;

fn options(path: PathBuf) -> CliRunOptions {
    CliRunOptions {
        path,
        dry_run: false,
        show_help: false,
        show_version: false,
        package_version: None,
        dependency_scope: String::new(),
    }
}

#[test]
fn run_with_missing_path_completes_without_error() {
    let path = std::env::temp_dir()
        .join("update-nuspec-run-tests")
        .join(support::uuid_dir_name())
        .join("missing");
    run(&options(path)).expect("missing path is reported, not fatal");
}

#[test]
fn run_with_package_version_updates_scoped_dependencies() {
    let workspace = support::copy_test_data();
    let package_json_path = workspace.path().join("package.json");

    run(&CliRunOptions {
        package_version: Some("4.0.0".to_string()),
        dependency_scope: "@guru/".to_string(),
        ..options(workspace.path().to_path_buf())
    })
    .expect("run");

    let json = fs::read_to_string(package_json_path).expect("read json");
    assert!(json.contains(r#""version": "4.0.0"#));
    assert!(json.contains(r#""@guru/core": "^4.0.0"#));
}

#[test]
fn run_without_package_json_when_version_set_completes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let nuspec_path = temp.path().join("Only.nuspec");
    fs::write(
        &nuspec_path,
        r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2010/07/nuspec.xsd">
  <metadata>
    <id>Only</id>
    <version>1.0.0</version>
    <dependencies />
  </metadata>
</package>"#,
    )
    .expect("write nuspec");

    run(&CliRunOptions {
        package_version: Some("1.2.3".to_string()),
        dependency_scope: "@guru/".to_string(),
        ..options(temp.path().to_path_buf())
    })
    .expect("run without package.json");
}

#[test]
fn run_skips_package_json_under_node_modules() {
    let temp = tempfile::tempdir().expect("tempdir");
    let node_modules = temp.path().join("node_modules").join("@guru");
    fs::create_dir_all(&node_modules).expect("create node_modules");
    fs::write(
        node_modules.join("package.json"),
        r#"{"version":"9.9.9","dependencies":{"@guru/core":"^9.9.9"}}"#,
    )
    .expect("write nested package.json");
    fs::write(
        temp.path().join("package.json"),
        r#"{"version":"1.0.0","dependencies":{"@guru/core":"^1.0.0"}}"#,
    )
    .expect("write root package.json");

    run(&CliRunOptions {
        package_version: Some("2.0.0".to_string()),
        dependency_scope: "@guru/".to_string(),
        ..options(temp.path().to_path_buf())
    })
    .expect("run");

    let root_json = fs::read_to_string(temp.path().join("package.json")).expect("read root");
    assert!(root_json.contains(r#""version": "2.0.0"#));
    assert!(root_json.contains(r#""@guru/core": "^2.0.0"#));

    let nested_json =
        fs::read_to_string(node_modules.join("package.json")).expect("read nested");
    assert!(nested_json.contains(r#""version":"9.9.9"#));
    assert!(nested_json.contains(r#""@guru/core":"^9.9.9"#));
}

#[test]
fn run_dry_run_reports_skipped_writes() {
    let workspace = support::copy_test_data();
    let nuspec_path = workspace.path().join("MyPackage.nuspec");
    let before = fs::read_to_string(&nuspec_path).expect("read before");

    run(&CliRunOptions {
        dry_run: true,
        ..options(workspace.path().to_path_buf())
    })
    .expect("run");

    assert_eq!(fs::read_to_string(&nuspec_path).expect("read after"), before);
}

#[test]
fn run_reports_project_file_not_found_for_orphan_nuspec() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(
        temp.path().join("Orphan.nuspec"),
        r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2010/07/nuspec.xsd">
  <metadata>
    <id>Orphan</id>
    <version>1.0.0</version>
    <dependencies />
  </metadata>
</package>"#,
    )
    .expect("write nuspec");

    run(&options(temp.path().to_path_buf())).expect("run");
}
