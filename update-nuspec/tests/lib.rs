mod support;

use std::collections::HashMap;
use std::fs;

use update_nuspec::{
    build_ordered_result_list, compare_dependencies, get_package_references_for_target_framework,
    process_nuspec, Dependency, DependencyComparisonResult, LibError, ProcessStatus,
};

#[test]
fn compare_dependencies_detects_added_updated_and_deleted() {
    let dependencies = vec![
        Dependency::new("A", "1.0.0"),
        Dependency::new("B", "2.0.0"),
        Dependency::new("Removed", "1.0.0"),
    ];
    let package_references = vec![
        Dependency::new("A", "1.0.0"),
        Dependency::new("B", "2.1.0"),
        Dependency::new("Added", "3.0.0"),
    ];

    let result = compare_dependencies(&dependencies, &package_references);

    assert_eq!(result.no_changes_references, vec![Dependency::new("A", "1.0.0")]);
    assert_eq!(result.updated_references, vec![Dependency::new("B", "2.1.0")]);
    assert_eq!(result.added_references, vec![Dependency::new("Added", "3.0.0")]);
    assert_eq!(result.deleted_references, vec![Dependency::new("Removed", "1.0.0")]);
}

#[test]
fn build_ordered_result_list_sorts_special_groups_first() {
    let comparison = DependencyComparisonResult {
        updated_references: vec![Dependency::new("Zebra", "1.0.0")],
        added_references: vec![
            Dependency::new("Cross.A", "1.0.0"),
            Dependency::new("Boilerplate.X", "1.0.0"),
            Dependency::new("My.Api.Contract", "1.0.0"),
        ],
        no_changes_references: vec![Dependency::new("Alpha", "1.0.0")],
        deleted_references: Vec::new(),
        outdated_references: HashMap::new(),
    };

    let ordered = build_ordered_result_list(&comparison);
    let names: Vec<_> = ordered
        .iter()
        .map(|dependency| dependency.name.as_str())
        .collect();

    assert_eq!(
        names,
        vec![
            "Cross.A",
            "Boilerplate.X",
            "My.Api.Contract",
            "Alpha",
            "Zebra"
        ]
    );
}

#[test]
fn my_package_syncs_newtonsoft_json_from_csproj() {
    let workspace = support::copy_test_data();
    let nuspec_path = workspace.path().join("MyPackage.nuspec");

    let result = process_nuspec(&nuspec_path, workspace.path(), false).expect("process");
    assert_eq!(result.status, ProcessStatus::Completed);

    let versions = support::dependency_versions(&nuspec_path, "Newtonsoft.Json");
    assert!(versions.iter().all(|version| version == "13.0.3"));
}

#[test]
fn cross_messaging_syncs_packages_per_target_framework_from_csproj() {
    let workspace = support::copy_test_data();
    let nuspec_path = workspace.path().join("Cross.Messaging.nuspec");

    let result = process_nuspec(&nuspec_path, workspace.path(), false).expect("process");
    assert_eq!(result.status, ProcessStatus::Completed);

    let mailkit_versions = support::dependency_versions(&nuspec_path, "MailKit");
    assert!(mailkit_versions.iter().all(|version| version == "4.16.0"));

    assert_eq!(
        support::dependency_version_in_group(&nuspec_path, "net8.0", "Microsoft.Extensions.Configuration"),
        Some("8.0.0".to_string())
    );
    assert_eq!(
        support::dependency_version_in_group(&nuspec_path, "net9.0", "Microsoft.Extensions.Configuration"),
        Some("9.0.15".to_string())
    );
    assert_eq!(
        support::dependency_version_in_group(&nuspec_path, "net10.0", "Microsoft.Extensions.Configuration"),
        Some("10.0.7".to_string())
    );

    assert!(!support::contains_dependency(
        &nuspec_path,
        "Microsoft.Extensions.Options.TTTT"
    ));
    assert!(!support::contains_dependency(
        &nuspec_path,
        "Microsoft.Extensions.Options.RRRRRR"
    ));

    assert_eq!(
        support::dependency_version_in_group(&nuspec_path, "net7.0", "MailKit"),
        Some("4.16.0".to_string())
    );
    assert_eq!(
        support::dependency_version_in_group(
            &nuspec_path,
            "net6.0",
            "Microsoft.Extensions.Configuration.Binder"
        ),
        Some("8.0.2".to_string())
    );
    assert_eq!(
        support::dependency_version_in_group(
            &nuspec_path,
            "net9.0",
            "Microsoft.Extensions.Configuration.Binder"
        ),
        None
    );
}

#[test]
fn boilerplate_data_filter_syncs_packages_from_csproj() {
    let workspace = support::copy_test_data();
    let nuspec_path = workspace.path().join("config.nuspec");

    let result = process_nuspec(&nuspec_path, workspace.path(), false).expect("process");
    assert_eq!(result.status, ProcessStatus::Completed);

    assert_eq!(
        support::dependency_versions(&nuspec_path, "Cross.CQRS.EF"),
        vec!["7.0.0".to_string()]
    );
    assert_eq!(
        support::dependency_versions(&nuspec_path, "Microsoft.EntityFrameworkCore"),
        vec!["6.123.47687".to_string()]
    );
    assert_eq!(
        support::dependency_versions(&nuspec_path, "Boilerplate.WebApi.Contract"),
        vec!["13.5.77".to_string()]
    );
    assert!(support::contains_dependency(
        &nuspec_path,
        "Microsoft.AspNetCore.Authentication.JwtBearer"
    ));
    assert!(!support::contains_dependency(
        &nuspec_path,
        "AspNetCore.HealthChecks.Rabbitmq"
    ));
}

#[test]
fn csproj_resolver_matches_cross_messaging_fixture() {
    let project_path = support::test_data_dir().join("Cross.Messaging.csproj");
    let project_xml = fs::read_to_string(project_path).expect("read csproj");

    let net8 = get_package_references_for_target_framework(&project_xml, "net8.0")
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

    let net9 = get_package_references_for_target_framework(&project_xml, "net9.0")
        .expect("net9 packages");
    assert!(!net9
        .iter()
        .any(|dependency| dependency.name == "Microsoft.Extensions.Configuration.Binder"));
    assert!(net9.iter().any(|dependency| {
        dependency.name == "Microsoft.Extensions.Configuration" && dependency.version == "9.0.15"
    }));

    let net10 = get_package_references_for_target_framework(&project_xml, "net10.0")
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
    assert!(matches!(error, LibError::InvalidXml { .. }));
}

#[test]
fn dry_run_does_not_modify_nuspec() {
    let workspace = support::copy_test_data();
    let nuspec_path = workspace.path().join("MyPackage.nuspec");
    let before = fs::read_to_string(&nuspec_path).expect("read before");

    let result = process_nuspec(&nuspec_path, workspace.path(), true).expect("process");
    assert_eq!(result.status, ProcessStatus::Completed);

    let after = fs::read_to_string(&nuspec_path).expect("read after");
    assert_eq!(before, after);
}
