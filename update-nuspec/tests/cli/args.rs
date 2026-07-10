use std::path::PathBuf;

use update_nuspec::cli::args::{
    parse_args, resolve_dependency_scope, resolve_package_version,
};

#[test]
fn parse_args_reads_package_version_and_scope() {
    let options = parse_args(&[
        "./dist".to_string(),
        "--package-version".to_string(),
        "1.2.3".to_string(),
        "--dependency-scope".to_string(),
        "@guru/".to_string(),
        "--dry-run".to_string(),
    ]);

    assert_eq!(options.path, PathBuf::from("./dist"));
    assert_eq!(options.package_version.as_deref(), Some("1.2.3"));
    assert_eq!(options.dependency_scope, "@guru/");
    assert!(options.dry_run);
}

#[test]
fn parse_args_reads_inline_option_values() {
    let options = parse_args(&[
        "./dist".to_string(),
        "-pv=2.0.0".to_string(),
        "-ds=".to_string(),
    ]);

    assert_eq!(options.package_version.as_deref(), Some("2.0.0"));
    assert_eq!(options.dependency_scope, "");
}

#[test]
fn resolve_package_version_prefers_cli_over_env() {
    temp_env::with_var("PACKAGE_VERSION", Some("9.9.9"), || {
        let result = resolve_package_version(Some("1.0.0"));
        assert_eq!(result.as_deref(), Some("1.0.0"));
    });
}

#[test]
fn resolve_package_version_reads_gitversion_env() {
    temp_env::with_var("GitVersion_SemVer", Some("1.3.0-preview.4"), || {
        let result = resolve_package_version(None);
        assert_eq!(result.as_deref(), Some("1.3.0-preview.4"));
    });
}

#[test]
fn resolve_dependency_scope_uses_empty_default_when_not_provided() {
    temp_env::with_var("DEPENDENCY_SCOPE", None::<&str>, || {
        assert_eq!(resolve_dependency_scope(None, false), "");
    });
}

#[test]
fn parse_args_treats_positional_true_as_dry_run() {
    let options = parse_args(&["./data".to_string(), "true".to_string()]);
    assert!(options.dry_run);
    assert_eq!(options.path, PathBuf::from("./data"));
}
