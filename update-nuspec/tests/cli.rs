use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn test_data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("UpdateNuspecTool.Tests/TestData")
}

fn run(args: &[&str]) -> (i32, String) {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("update-nuspec")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run binary");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let combined = if stderr.is_empty() {
        stdout
    } else {
        format!("{stdout}{stderr}")
    };

    (output.status.code().unwrap_or(-1), combined)
}

fn copy_test_data() -> tempfile::TempDir {
    let source = test_data_dir();
    let temp = tempfile::tempdir().expect("tempdir");
    copy_dir_all(&source, temp.path()).expect("copy test data");
    temp
}

fn copy_dir_all(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[test]
fn main_with_help_prints_usage() {
    let (_, output) = run(&["--help"]);
    assert!(output.contains("USAGE"));
    assert!(output.contains("update-nuspec"));
}

#[test]
fn main_with_version_prints_version_line() {
    let (_, output) = run(&["--version"]);
    assert!(output.contains("update-nuspec"));
    assert!(output.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn main_with_invalid_path_prints_error() {
    let path = std::env::temp_dir()
        .join("update-nuspec-cli-tests")
        .join(uuid_dir_name())
        .join("missing");
    let (_, output) = run(&[path.to_str().expect("path")]);
    assert!(output.contains("is not valid"));
}

#[test]
fn main_with_empty_directory_prints_nuspec_not_found() {
    let temp = tempfile::tempdir().expect("tempdir");
    let (_, output) = run(&[temp.path().to_str().expect("path")]);
    assert!(output.contains("*.nuspec files not found"));
}

#[test]
fn main_with_dry_run_does_not_modify_nuspec() {
    let workspace = copy_test_data();
    let nuspec_path = workspace.path().join("MyPackage.nuspec");
    let before = fs::read_to_string(&nuspec_path).expect("read before");

    let (_, output) = run(&[
        workspace.path().to_str().expect("path"),
        "--dry-run",
    ]);

    assert!(output.contains("[DRY RUN]"));
    assert!(output.contains("Start processing file"));
    assert_eq!(fs::read_to_string(&nuspec_path).expect("read after"), before);
}

#[test]
fn main_processes_nuspec_in_workspace() {
    let workspace = copy_test_data();
    let nuspec_path = workspace.path().join("MyPackage.nuspec");

    let (code, output) = run(&[workspace.path().to_str().expect("path")]);
    assert_eq!(code, 0);
    assert!(output.contains("Start processing file"));

    let xml = fs::read_to_string(nuspec_path).expect("read nuspec");
    assert!(xml.contains(r#"version="13.0.3""#));
}

#[test]
fn main_with_package_version_updates_package_json() {
    let workspace = copy_test_data();
    let package_json_path = workspace.path().join("package.json");

    let (code, output) = run(&[
        workspace.path().to_str().expect("path"),
        "--package-version",
        "3.0.0",
    ]);

    assert_eq!(code, 0);
    assert!(output.contains("Start processing file"));
    assert!(output.contains("package.json"));

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(package_json_path).expect("read json"))
            .expect("parse json");
    assert_eq!(json["version"], "3.0.0");
}

fn uuid_dir_name() -> String {
    format!(
        "{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    )
}
