#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use roxmltree::Document;

pub fn test_data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("UpdateNuspecTool.Tests/TestData")
}

pub fn copy_test_data() -> tempfile::TempDir {
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

pub fn run_cli(args: &[&str]) -> (i32, String) {
    let binary = std::env::var("CARGO_BIN_EXE_update-nuspec")
        .expect("CARGO_BIN_EXE_update-nuspec is set when running integration tests");
    let output = Command::new(binary)
        .args(args)
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

pub fn uuid_dir_name() -> String {
    format!(
        "{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    )
}

pub fn dependency_versions(nuspec_path: &Path, package_id: &str) -> Vec<String> {
    let xml = fs::read_to_string(nuspec_path).expect("read nuspec");
    let document = Document::parse(&xml).expect("parse nuspec");

    document
        .descendants()
        .filter(|node| node.tag_name().name() == "dependency")
        .filter(|node| node.attribute("id") == Some(package_id))
        .filter_map(|node| node.attribute("version"))
        .map(str::to_string)
        .collect()
}

pub fn dependency_version_in_group(
    nuspec_path: &Path,
    target_framework: &str,
    package_id: &str,
) -> Option<String> {
    let xml = fs::read_to_string(nuspec_path).expect("read nuspec");
    let document = Document::parse(&xml).expect("parse nuspec");

    let group = document.descendants().find(|node| {
        node.tag_name().name() == "group"
            && node.attribute("targetFramework") == Some(target_framework)
    })?;

    group
        .children()
        .filter(|node| node.is_element())
        .find(|node| {
            node.tag_name().name() == "dependency" && node.attribute("id") == Some(package_id)
        })
        .and_then(|node| node.attribute("version"))
        .map(str::to_string)
}

pub fn contains_dependency(nuspec_path: &Path, package_id: &str) -> bool {
    !dependency_versions(nuspec_path, package_id).is_empty()
}
