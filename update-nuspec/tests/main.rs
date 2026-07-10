mod support;

use std::fs;

#[test]
fn main_with_help_prints_usage() {
    let (_, output) = support::run_cli(&["--help"]);
    assert!(output.contains("USAGE"));
    assert!(output.contains("update-nuspec"));
}

#[test]
fn main_with_version_prints_version_line() {
    let (_, output) = support::run_cli(&["--version"]);
    assert!(output.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn main_with_invalid_path_prints_error() {
    let path = std::env::temp_dir()
        .join("update-nuspec-cli-tests")
        .join(support::uuid_dir_name())
        .join("missing");
    let (_, output) = support::run_cli(&[path.to_str().expect("path")]);
    assert!(output.contains("is not valid"));
}

#[test]
fn main_with_empty_directory_prints_nuspec_not_found() {
    let temp = tempfile::tempdir().expect("tempdir");
    let (_, output) = support::run_cli(&[temp.path().to_str().expect("path")]);
    assert!(output.contains("*.nuspec files not found"));
}

#[test]
fn main_with_dry_run_does_not_modify_nuspec() {
    let workspace = support::copy_test_data();
    let nuspec_path = workspace.path().join("MyPackage.nuspec");
    let before = fs::read_to_string(&nuspec_path).expect("read before");

    let (_, output) = support::run_cli(&[
        workspace.path().to_str().expect("path"),
        "--dry-run",
    ]);

    assert!(output.contains("[DRY RUN]"));
    assert!(output.contains("Start processing file"));
    assert_eq!(fs::read_to_string(&nuspec_path).expect("read after"), before);
}

#[test]
fn main_processes_nuspec_in_workspace() {
    let workspace = support::copy_test_data();
    let nuspec_path = workspace.path().join("MyPackage.nuspec");

    let (code, output) = support::run_cli(&[workspace.path().to_str().expect("path")]);
    assert_eq!(code, 0);
    assert!(output.contains("Start processing file"));

    let xml = fs::read_to_string(nuspec_path).expect("read nuspec");
    assert!(xml.contains(r#"version="13.0.3""#));
}

#[test]
fn main_with_package_version_updates_package_json() {
    let workspace = support::copy_test_data();
    let package_json_path = workspace.path().join("package.json");

    let (code, output) = support::run_cli(&[
        workspace.path().to_str().expect("path"),
        "--package-version",
        "3.0.0",
    ]);

    assert_eq!(code, 0);
    assert!(output.contains("Start processing file"));
    assert!(output.contains("package.json"));

    let json = fs::read_to_string(package_json_path).expect("read json");
    assert!(json.contains(r#""version": "3.0.0"#));
}

#[test]
fn main_with_dependency_scope_updates_scoped_dependencies() {
    let workspace = support::copy_test_data();
    let package_json_path = workspace.path().join("package.json");

    let (code, output) = support::run_cli(&[
        workspace.path().to_str().expect("path"),
        "--package-version",
        "5.0.0",
        "--dependency-scope",
        "@guru/",
        "--dry-run",
    ]);

    assert_eq!(code, 0);
    assert!(output.contains("package.json"));
    assert!(output.contains("Updated scoped dependencies"));

    let json = fs::read_to_string(package_json_path).expect("read json");
    assert!(json.contains(r#""version": "1.0.0"#));
    assert!(json.contains(r#""@guru/core": "^1.0.0"#));
}

#[test]
fn main_returns_error_when_nuspec_is_missing_dependencies() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(
        temp.path().join("Broken.nuspec"),
        r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2010/07/nuspec.xsd">
  <metadata>
    <id>Broken</id>
    <version>1.0.0</version>
  </metadata>
</package>"#,
    )
    .expect("write nuspec");
    fs::write(
        temp.path().join("Broken.csproj"),
        r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>"#,
    )
    .expect("write csproj");

    let (code, output) = support::run_cli(&[temp.path().to_str().expect("path")]);
    assert_eq!(code, 1);
    assert!(output.contains("missing <dependencies>"));
}
