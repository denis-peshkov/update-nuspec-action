use std::fs;

use update_nuspec::cli::console::Console;
use update_nuspec::cli::package_json;
use update_nuspec::LibError;

const FIXTURE: &str = r#"{
  "name": "@guru/my-package",
  "version": "1.0.0",
  "dependencies": {
    "@guru/core": "^1.0.0",
    "lodash": "4.17.21"
  },
  "devDependencies": {
    "@guru/dev-tools": "^0.9.0"
  },
  "peerDependencies": {
    "@guru/shared": "1.0.0"
  },
  "optionalDependencies": {
    "@guru/optional": "~1.0.0"
  }
}"#;

#[test]
fn updates_version_and_scoped_dependencies_in_all_sections() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("package.json");
    fs::write(&path, FIXTURE).expect("write fixture");

    package_json::process(&path, "2.0.0", "@guru/", false, &Console::new(false)).expect("process");

    let json = fs::read_to_string(path).expect("read json");
    assert!(json.contains(r#""version": "2.0.0"#));
    assert!(json.contains(r#""@guru/core": "^2.0.0"#));
    assert!(json.contains(r#""@guru/dev-tools": "^2.0.0"#));
    assert!(json.contains(r#""@guru/shared": "^2.0.0"#));
    assert!(json.contains(r#""@guru/optional": "^2.0.0"#));
    assert!(json.contains(r#""lodash": "4.17.21"#));
}

#[test]
fn dry_run_leaves_package_json_unchanged() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("package.json");
    fs::write(&path, FIXTURE).expect("write fixture");
    let before = fs::read_to_string(&path).expect("read before");

    package_json::process(&path, "2.0.0", "@guru/", true, &Console::new(true)).expect("process");

    assert_eq!(fs::read_to_string(&path).expect("read after"), before);
}

#[test]
fn empty_scope_updates_only_version() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("package.json");
    fs::write(&path, FIXTURE).expect("write fixture");

    package_json::process(&path, "2.0.0", "", false, &Console::new(false)).expect("process");

    let json = fs::read_to_string(path).expect("read json");
    assert!(json.contains(r#""version": "2.0.0"#));
    assert!(json.contains(r#""@guru/core": "^1.0.0"#));
    assert!(json.contains(r#""@guru/dev-tools": "^0.9.0"#));
}

#[test]
fn unchanged_version_still_aligns_scoped_dependencies() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("package.json");
    fs::write(&path, FIXTURE).expect("write fixture");

    package_json::process(&path, "1.0.0", "@guru/", false, &Console::new(false)).expect("process");

    let json = fs::read_to_string(path).expect("read json");
    assert!(json.contains(r#""version": "1.0.0"#));
    assert!(json.contains(r#""@guru/core": "^1.0.0"#));
    assert!(json.contains(r#""@guru/dev-tools": "^1.0.0"#));
    assert!(json.contains(r#""@guru/shared": "^1.0.0"#));
    assert!(json.contains(r#""@guru/optional": "^1.0.0"#));
}

#[test]
fn skips_dependencies_already_at_target_version() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("package.json");
    fs::write(
        &path,
        r#"{
  "version": "2.0.0",
  "dependencies": {
    "@guru/core": "^2.0.0"
  }
}"#,
    )
    .expect("write fixture");
    let before = fs::read_to_string(&path).expect("read before");

    package_json::process(&path, "2.0.0", "@guru/", true, &Console::new(true)).expect("process");

    assert_eq!(fs::read_to_string(&path).expect("read after dry run"), before);

    package_json::process(&path, "2.0.0", "@guru/", false, &Console::new(false)).expect("process");

    let root: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).expect("read json"))
        .expect("parse json");
    assert_eq!(root["version"], "2.0.0");
    assert_eq!(root["dependencies"]["@guru/core"], "^2.0.0");
}

#[test]
fn invalid_json_returns_error() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("package.json");
    fs::write(&path, "not-json").expect("write fixture");

    let error = package_json::process(&path, "1.0.0", "@guru/", false, &Console::new(false))
        .expect_err("invalid json");
    assert!(matches!(error, LibError::InvalidXml { .. }));
}

#[test]
fn non_object_root_returns_without_writing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("package.json");
    fs::write(&path, "[1, 2, 3]").expect("write fixture");
    let before = fs::read_to_string(&path).expect("read before");

    package_json::process(&path, "2.0.0", "@guru/", false, &Console::new(false)).expect("process");

    assert_eq!(fs::read_to_string(&path).expect("read after"), before);
}

#[test]
fn missing_file_returns_read_error() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("missing-package.json");

    let error = package_json::process(&path, "1.0.0", "@guru/", false, &Console::new(false))
        .expect_err("missing file");
    assert!(matches!(error, LibError::ReadFile { .. }));
}
