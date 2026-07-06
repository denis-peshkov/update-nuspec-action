use std::fs;
use std::path::Path;

use serde_json::{Map, Value};

use crate::cli::console::{Color, Console};
use update_nuspec::LibError;

const DEPENDENCY_SECTIONS: [&str; 4] = [
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];

struct PackageJsonChange {
    section: String,
    name: String,
    old_version: String,
    new_version: String,
}

impl Clone for PackageJsonChange {
    fn clone(&self) -> Self {
        Self {
            section: self.section.clone(),
            name: self.name.clone(),
            old_version: self.old_version.clone(),
            new_version: self.new_version.clone(),
        }
    }
}

pub fn process(
    file_path: &Path,
    package_version: &str,
    dependency_scope_prefix: &str,
    dry_run: bool,
    console: &Console,
) -> Result<(), LibError> {
    let json = fs::read_to_string(file_path).map_err(|source| LibError::ReadFile {
        path: file_path.to_path_buf(),
        source,
    })?;

    let mut root: Value = serde_json::from_str(&json).map_err(|error| LibError::InvalidXml {
        path: file_path.to_path_buf(),
        message: error.to_string(),
    })?;

    let Some(root_object) = root.as_object_mut() else {
        console.write_line(
            &format!("Invalid package.json: {}", file_path.display()),
            Color::Red,
        );
        return Ok(());
    };

    let old_version = root_object
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let version_changed = old_version != package_version;

    console.write("Current version: ", Color::Gray);
    console.write_line(&old_version, Color::Cyan);
    console.write("New version: ", Color::Gray);
    console.write_line(package_version, Color::Cyan);

    let dependency_changes =
        align_scoped_dependencies(root_object, package_version, dependency_scope_prefix);

    if version_changed {
        root_object.insert(
            "version".to_string(),
            Value::String(package_version.to_string()),
        );
    }

    show_result(
        console,
        version_changed,
        &old_version,
        package_version,
        &dependency_changes,
    );

    if !dry_run {
        let output = format!(
            "{}\n",
            serde_json::to_string_pretty(&root).map_err(|error| LibError::InvalidXml {
                path: file_path.to_path_buf(),
                message: error.to_string(),
            })?
        );
        fs::write(file_path, output).map_err(|source| LibError::WriteFile {
            path: file_path.to_path_buf(),
            source,
        })?;
    }

    Ok(())
}

fn align_scoped_dependencies(
    root: &mut Map<String, Value>,
    package_version: &str,
    dependency_scope_prefix: &str,
) -> Vec<PackageJsonChange> {
    let mut changes = Vec::new();

    if dependency_scope_prefix.is_empty() {
        return changes;
    }

    let new_version = format!("^{package_version}");

    for section in DEPENDENCY_SECTIONS {
        let Some(section_value) = root.get_mut(section) else {
            continue;
        };
        let Some(section_object) = section_value.as_object_mut() else {
            continue;
        };

        let keys: Vec<String> = section_object.keys().cloned().collect();
        for key in keys {
            if !key.starts_with(dependency_scope_prefix) {
                continue;
            }

            let old_value = section_object
                .get(&key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();

            if old_value == new_version {
                continue;
            }

            section_object.insert(key.clone(), Value::String(new_version.clone()));
            changes.push(PackageJsonChange {
                section: section.to_string(),
                name: key,
                old_version: old_value,
                new_version: new_version.clone(),
            });
        }
    }

    changes
}

fn show_result(
    console: &Console,
    version_changed: bool,
    old_version: &str,
    new_version: &str,
    dependency_changes: &[PackageJsonChange],
) {
    if version_changed {
        console.write("\t Version: ", Color::Gray);
        console.write_line(&format!("{old_version} -> {new_version}"), Color::Yellow);
    } else {
        console.write_line("\t Version: not changed", Color::Gray);
    }

    if dependency_changes.is_empty() {
        console.write_line("\t Scoped dependencies: not changed", Color::Gray);
        println!();
        return;
    }

    console.write_line(
        &format!(
            "\t Updated scoped dependencies {}:",
            dependency_changes.len()
        ),
        Color::Yellow,
    );

    let mut sorted_changes = dependency_changes.to_vec();
    sorted_changes.sort_by(|left, right| {
        left.section
            .cmp(&right.section)
            .then_with(|| left.name.cmp(&right.name))
    });

    for change in sorted_changes {
        console.write(&format!("\t\t [{}] ", change.section), Color::Gray);
        console.write(&format!("{}: ", change.name), Color::Yellow);
        console.write(&format!("{} ", change.old_version), Color::Gray);
        console.write("-> ", Color::Gray);
        console.write_line(&change.new_version, Color::Yellow);
    }

    println!();
}
