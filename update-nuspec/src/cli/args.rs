use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliRunOptions {
    pub path: PathBuf,
    pub dry_run: bool,
    pub show_help: bool,
    pub show_version: bool,
    pub package_version: Option<String>,
    pub dependency_scope: String,
}

pub fn parse_args(args: &[String]) -> CliRunOptions {
    let mut path = None;
    let mut dry_run = false;
    let mut show_help = false;
    let mut show_version = false;
    let mut package_version = None;
    let mut dependency_scope = None;
    let mut dependency_scope_provided = false;

    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];

        if is_help_switch(arg) {
            show_help = true;
            index += 1;
            continue;
        }

        if is_version_switch(arg) {
            show_version = true;
            index += 1;
            continue;
        }

        if is_dry_run_switch(arg) {
            dry_run = true;
            index += 1;
            continue;
        }

        if is_option(arg, "--package-version", "-pv") {
            package_version = Some(read_option_value(args, &mut index, arg));
            index += 1;
            continue;
        }

        if is_option(arg, "--dependency-scope", "-ds") {
            dependency_scope = Some(read_option_value(args, &mut index, arg));
            dependency_scope_provided = true;
            index += 1;
            continue;
        }

        if path.is_none() && !arg.starts_with('-') {
            path = Some(PathBuf::from(arg));
        }

        index += 1;
    }

    CliRunOptions {
        path: path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        dry_run,
        show_help,
        show_version,
        package_version: resolve_package_version(package_version.as_deref()),
        dependency_scope: resolve_dependency_scope(dependency_scope.as_deref(), dependency_scope_provided),
    }
}

pub fn resolve_package_version(cli_value: Option<&str>) -> Option<String> {
    if let Some(value) = cli_value.map(str::trim).filter(|value| !value.is_empty()) {
        return Some(value.to_string());
    }

    for name in [
        "PACKAGE_VERSION",
        "GITVERSION_SEMVER",
        "GitVersion_SemVer",
        "semVer",
        "SEMVER",
    ] {
        if let Ok(value) = env::var(name) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

pub fn resolve_dependency_scope(cli_value: Option<&str>, cli_provided: bool) -> String {
    if cli_provided {
        return cli_value.unwrap_or("").to_string();
    }

    env::var("DEPENDENCY_SCOPE").unwrap_or_default()
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn help_text() -> String {
    format!(
        r#"update-nuspec {version}
Sync NuGet <dependencies> in *.nuspec with PackageReference versions from matching *.csproj files.
Optionally update package.json version and scoped npm dependencies.

DESCRIPTION
  Recursively scans a directory for .nuspec files, finds a .csproj with the same name as <id>
  in nuspec metadata in each file's folder, compares package versions, and rewrites the
  <dependencies> section (flat or per targetFramework group).
  When --package-version (or PACKAGE_VERSION / GitVersion_SemVer / SemVer env) is set, also updates
  package.json: sets version as x.y.z and aligns dependencies whose names start with the scope prefix
  to ^x.y.z in dependencies, devDependencies, peerDependencies, optionalDependencies when --dependency-scope is set.
  Use dry-run to preview changes without saving files.

USAGE
  update-nuspec [path] [options]

ARGUMENTS
  path                    Root directory to scan recursively (default: current directory).
                          In GitHub Actions the action passes a path relative to /github/workspace.

OPTIONS
  --help, -h, -?          Show this help.
  --version, -v           Show tool version.
  --dry-run, -d, --demo   Analyze and print the report; do not modify files.
  true                    Same as --dry-run (positional boolean).
  --package-version, -pv  SemVer for package.json "version" (env: PACKAGE_VERSION, GitVersion_SemVer, semVer).
  --dependency-scope, -ds Scope prefix for npm dependency alignment (env: DEPENDENCY_SCOPE).
                          Skipped when empty.

EXAMPLES
  update-nuspec
  update-nuspec ./src/MyPackage
  update-nuspec ./client/dist/my-app --package-version 1.2.3
  update-nuspec ./client/dist/my-app -pv 1.2.3-preview.4 -ds @guru/
  update-nuspec ./UpdateNuspecTool.Tests/TestData --dry-run
  update-nuspec -d .
  update-nuspec --version
  update-nuspec --help

GITHUB ACTION
  - uses: denis-peshkov/update-nuspec-action@v1
    with:
      dir: client/dist/my-app
      packageVersion: 1.2.3
"#,
        version = version()
    )
}

fn is_dry_run_switch(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--dry-run")
        || arg.eq_ignore_ascii_case("-d")
        || arg.eq_ignore_ascii_case("--demo")
        || arg.eq_ignore_ascii_case("true")
}

fn is_help_switch(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--help")
        || arg.eq_ignore_ascii_case("-h")
        || arg.eq_ignore_ascii_case("-?")
        || arg.eq_ignore_ascii_case("/?")
}

fn is_version_switch(arg: &str) -> bool {
    arg.eq_ignore_ascii_case("--version") || arg.eq_ignore_ascii_case("-v")
}

fn is_option(arg: &str, long_name: &str, short_name: &str) -> bool {
    arg.eq_ignore_ascii_case(long_name)
        || arg.eq_ignore_ascii_case(short_name)
        || arg
            .get(..long_name.len() + 1)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(&format!("{long_name}=")))
        || arg
            .get(..short_name.len() + 1)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(&format!("{short_name}=")))
}

fn read_option_value(args: &[String], index: &mut usize, arg: &str) -> String {
    if let Some((_, value)) = arg.split_once('=') {
        return value.to_string();
    }

    let next_index = *index + 1;
    if next_index >= args.len() {
        panic!("Missing value for option '{arg}'.");
    }

    *index = next_index;
    args[next_index].clone()
}

