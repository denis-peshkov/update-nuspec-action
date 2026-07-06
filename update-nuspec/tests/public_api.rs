use std::fs;
use std::path::{Path, PathBuf};

use update_nuspec::{process_nuspec, ProcessStatus};
use roxmltree::Document;

fn test_data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("UpdateNuspecTool.Tests/TestData")
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

fn dependency_versions(nuspec_path: &Path, package_id: &str) -> Vec<String> {
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

fn dependency_version_in_group(
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

fn contains_dependency(nuspec_path: &Path, package_id: &str) -> bool {
    !dependency_versions(nuspec_path, package_id).is_empty()
}

#[test]
fn my_package_syncs_newtonsoft_json_from_csproj() {
    let workspace = copy_test_data();
    let nuspec_path = workspace.path().join("MyPackage.nuspec");

    let result = process_nuspec(&nuspec_path, workspace.path(), false).expect("process");
    assert_eq!(result.status, ProcessStatus::Completed);

    let versions = dependency_versions(&nuspec_path, "Newtonsoft.Json");
    assert!(versions.iter().all(|version| version == "13.0.3"));
}

#[test]
fn cross_messaging_syncs_packages_per_target_framework_from_csproj() {
    let workspace = copy_test_data();
    let nuspec_path = workspace.path().join("Cross.Messaging.nuspec");

    let result = process_nuspec(&nuspec_path, workspace.path(), false).expect("process");
    assert_eq!(result.status, ProcessStatus::Completed);

    let mailkit_versions = dependency_versions(&nuspec_path, "MailKit");
    assert!(mailkit_versions.iter().all(|version| version == "4.16.0"));

    assert_eq!(
        dependency_version_in_group(&nuspec_path, "net8.0", "Microsoft.Extensions.Configuration"),
        Some("8.0.0".to_string())
    );
    assert_eq!(
        dependency_version_in_group(&nuspec_path, "net9.0", "Microsoft.Extensions.Configuration"),
        Some("9.0.15".to_string())
    );
    assert_eq!(
        dependency_version_in_group(&nuspec_path, "net10.0", "Microsoft.Extensions.Configuration"),
        Some("10.0.7".to_string())
    );

    assert!(!contains_dependency(
        &nuspec_path,
        "Microsoft.Extensions.Options.TTTT"
    ));
    assert!(!contains_dependency(
        &nuspec_path,
        "Microsoft.Extensions.Options.RRRRRR"
    ));

    assert_eq!(
        dependency_version_in_group(&nuspec_path, "net7.0", "MailKit"),
        Some("4.16.0".to_string())
    );
    assert_eq!(
        dependency_version_in_group(
            &nuspec_path,
            "net6.0",
            "Microsoft.Extensions.Configuration.Binder"
        ),
        Some("8.0.2".to_string())
    );
    assert_eq!(
        dependency_version_in_group(
            &nuspec_path,
            "net9.0",
            "Microsoft.Extensions.Configuration.Binder"
        ),
        None
    );
}

#[test]
fn boilerplate_data_filter_syncs_packages_from_csproj() {
    let workspace = copy_test_data();
    let nuspec_path = workspace.path().join("config.nuspec");

    let result = process_nuspec(&nuspec_path, workspace.path(), false).expect("process");
    assert_eq!(result.status, ProcessStatus::Completed);

    assert_eq!(
        dependency_versions(&nuspec_path, "Cross.CQRS.EF"),
        vec!["7.0.0".to_string()]
    );
    assert_eq!(
        dependency_versions(&nuspec_path, "Microsoft.EntityFrameworkCore"),
        vec!["6.123.47687".to_string()]
    );
    assert_eq!(
        dependency_versions(&nuspec_path, "Boilerplate.WebApi.Contract"),
        vec!["13.5.77".to_string()]
    );
    assert!(contains_dependency(
        &nuspec_path,
        "Microsoft.AspNetCore.Authentication.JwtBearer"
    ));
    assert!(!contains_dependency(
        &nuspec_path,
        "AspNetCore.HealthChecks.Rabbitmq"
    ));
}

#[test]
fn csproj_resolver_matches_cross_messaging_fixture() {
    let project_path = test_data_dir().join("Cross.Messaging.csproj");
    let project_xml = fs::read_to_string(project_path).expect("read csproj");

    let net8 = update_nuspec::get_package_references_for_target_framework(&project_xml, "net8.0")
        .expect("net8 packages");
    assert!(net8.iter().any(|dependency| {
        dependency.name == "MailKit" && dependency.version == "4.16.0"
    }));
    assert!(net8.iter().any(|dependency| {
        dependency.name == "Microsoft.Extensions.Configuration" && dependency.version == "8.0.0"
    }));
    assert!(net8.iter().any(|dependency| {
        dependency.name == "Microsoft.Extensions.Configuration.Binder" && dependency.version == "8.0.2"
    }));
    assert!(!net8
        .iter()
        .any(|dependency| dependency.name == "Microsoft.SourceLink.GitHub"));

    let net9 = update_nuspec::get_package_references_for_target_framework(&project_xml, "net9.0")
        .expect("net9 packages");
    assert!(!net9
        .iter()
        .any(|dependency| dependency.name == "Microsoft.Extensions.Configuration.Binder"));
    assert!(net9.iter().any(|dependency| {
        dependency.name == "Microsoft.Extensions.Configuration" && dependency.version == "9.0.15"
    }));

    let net10 = update_nuspec::get_package_references_for_target_framework(&project_xml, "net10.0")
        .expect("net10 packages");
    assert!(net10.iter().any(|dependency| {
        dependency.name == "Microsoft.Extensions.Configuration" && dependency.version == "10.0.7"
    }));
}

#[test]
fn process_missing_metadata_id_returns_project_name_not_found() {
    let temp = tempfile::tempdir().expect("tempdir");
    let nuspec_path = temp.path().join("Broken.nuspec");
    fs::write(
        &nuspec_path,
        r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2010/07/nuspec.xsd">
  <metadata>
    <version>1.0.0</version>
    <dependencies />
  </metadata>
</package>"#,
    )
    .expect("write nuspec");

    let result = process_nuspec(&nuspec_path, temp.path(), true).expect("process");
    assert_eq!(result.status, ProcessStatus::ProjectNameNotFound);
}

#[test]
fn process_missing_csproj_returns_project_file_not_found() {
    let temp = tempfile::tempdir().expect("tempdir");
    let nuspec_path = temp.path().join("Orphan.nuspec");
    fs::write(
        &nuspec_path,
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

    let result = process_nuspec(&nuspec_path, temp.path(), true).expect("process");
    assert_eq!(result.status, ProcessStatus::ProjectFileNotFound);
}

#[test]
fn process_invalid_xml_returns_error() {
    let temp = tempfile::tempdir().expect("tempdir");
    let nuspec_path = temp.path().join("Bad.nuspec");
    fs::write(&nuspec_path, "<not-valid-xml").expect("write nuspec");

    let error = process_nuspec(&nuspec_path, temp.path(), true).expect_err("invalid xml");
    assert!(matches!(error, update_nuspec::LibError::InvalidXml { .. }));
}

#[test]
fn dry_run_does_not_modify_nuspec() {
    let workspace = copy_test_data();
    let nuspec_path = workspace.path().join("MyPackage.nuspec");
    let before = fs::read_to_string(&nuspec_path).expect("read before");

    let result = process_nuspec(&nuspec_path, workspace.path(), true).expect("process");
    assert_eq!(result.status, ProcessStatus::Completed);

    let after = fs::read_to_string(&nuspec_path).expect("read after");
    assert_eq!(before, after);
}
