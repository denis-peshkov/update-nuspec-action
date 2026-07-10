//! CLI and library to sync NuGet `<dependencies>` in `*.nuspec` with `PackageReference`
//! versions from matching `*.csproj` files.
//!
//! The `update-nuspec` binary scans directories recursively, compares dependency versions,
//! and rewrites nuspec XML. Optional `package.json` version and scoped npm dependency
//! alignment is handled by the CLI entry point (`src/cli/`).

mod error;
mod parser;
mod types;

pub use error::LibError;
pub use parser::{
    condition_applies_to_target_framework, get_package_references,
    get_package_references_for_target_framework,
};
pub use types::{
    Dependency, DependencyComparisonResult, GroupComparisonResult, ProcessResult, ProcessStatus,
};

pub mod cli;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use xmltree::{Element, XMLNode};

/// Syncs NuGet `<dependencies>` in a `.nuspec` file with `PackageReference` versions from the
/// matching `.csproj` file in `project_dir`.
pub fn process_nuspec(
    nuspec_path: &Path,
    project_dir: &Path,
    dry_run: bool,
) -> Result<ProcessResult, LibError> {
    let nuspec_xml = read_to_string(nuspec_path)?;
    let mut root = parse_xml(nuspec_path, &nuspec_xml)?;

    let project_name = {
        let metadata = find_child_element(&root, "metadata").ok_or_else(|| LibError::InvalidXml {
            path: nuspec_path.to_path_buf(),
            message: "metadata element not found".to_string(),
        })?;

        find_child_element(metadata, "id")
            .and_then(element_text)
            .filter(|value| !value.is_empty())
    };

    let Some(project_name) = project_name else {
        return Ok(ProcessResult {
            status: ProcessStatus::ProjectNameNotFound,
            project_id: None,
            comparison: None,
            group_comparisons: Vec::new(),
        });
    };

    let project_path = project_dir.join(format!("{project_name}.csproj"));
    if !project_path.is_file() {
        return Ok(ProcessResult {
            status: ProcessStatus::ProjectFileNotFound,
            project_id: Some(project_name),
            comparison: None,
            group_comparisons: Vec::new(),
        });
    }

    let project_xml = read_to_string(&project_path)?;

    let mut group_comparisons = Vec::new();
    let mut flat_comparison = None;

    let group_targets: Vec<String> = {
        let metadata = find_child_element(&root, "metadata").expect("metadata");
        let dependencies_element = find_child_element(metadata, "dependencies").ok_or_else(|| {
            LibError::MissingDependencies {
                path: nuspec_path.to_path_buf(),
            }
        })?;

        dependencies_element
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| element.name == "group")
            .map(|group| {
                group
                    .attributes
                    .get("targetFramework")
                    .cloned()
                    .unwrap_or_default()
            })
            .collect()
    };

    if !group_targets.is_empty() {
        for target_framework in group_targets {
            let group_dependencies = {
                let metadata = find_child_element(&root, "metadata").expect("metadata");
                let dependencies_element =
                    find_child_element(metadata, "dependencies").expect("dependencies");
                let group = dependencies_element
                    .children
                    .iter()
                    .filter_map(XMLNode::as_element)
                    .find(|element| {
                        element.name == "group"
                            && element
                                .attributes
                                .get("targetFramework")
                                .map(String::as_str)
                                .unwrap_or_default()
                                == target_framework
                    })
                    .expect("group");
                get_group_dependencies(group)
            };

            let package_references = get_package_references_for_target_framework(
                &project_xml,
                &target_framework,
            )
            .map_err(|message| LibError::InvalidXml {
                path: project_path.clone(),
                message,
            })?;

            let comparison = compare_dependencies(&group_dependencies, &package_references);
            group_comparisons.push(GroupComparisonResult {
                target_framework: if target_framework.trim().is_empty() {
                    "(unknown)".to_string()
                } else {
                    target_framework.clone()
                },
                comparison: comparison.clone(),
            });

            if !dry_run {
                let result_list = build_ordered_result_list(&comparison);
                apply_dependencies_to_single_group(&mut root, &target_framework, &result_list);
            }
        }
    } else {
        let dependencies = {
            let metadata = find_child_element(&root, "metadata").expect("metadata");
            let dependencies_element = find_child_element(metadata, "dependencies").ok_or_else(|| {
                LibError::MissingDependencies {
                    path: nuspec_path.to_path_buf(),
                }
            })?;

            dependencies_element
                .children
                .iter()
                .filter_map(XMLNode::as_element)
                .filter(|element| element.name == "dependency")
                .filter_map(dependency_from_element)
                .collect::<Vec<_>>()
        };

        let package_references = get_package_references(&project_xml).map_err(|message| {
            LibError::InvalidXml {
                path: project_path.clone(),
                message,
            }
        })?;

        let comparison = compare_dependencies(&dependencies, &package_references);
        flat_comparison = Some(comparison.clone());

        if !dry_run {
            let result_list = build_ordered_result_list(&comparison);
            replace_flat_dependencies(&mut root, &result_list);
        }
    }

    if !dry_run {
        write_xml(nuspec_path, &root)?;
    }

    Ok(ProcessResult {
        status: ProcessStatus::Completed,
        project_id: Some(project_name),
        comparison: flat_comparison,
        group_comparisons,
    })
}

pub fn compare_dependencies(
    dependencies: &[Dependency],
    package_references: &[Dependency],
) -> DependencyComparisonResult {
    let mut result = DependencyComparisonResult::default();

    for item in package_references {
        let dependency_to_update: Vec<_> = dependencies
            .iter()
            .filter(|dependency| dependency.name == item.name)
            .collect();

        if dependency_to_update.is_empty() {
            result.added_references.push(item.clone());
            continue;
        }

        for dependency in dependency_to_update {
            if dependency.version != item.version {
                result
                    .outdated_references
                    .insert(dependency.name.clone(), dependency.version.clone());
                result.updated_references.push(item.clone());
            } else {
                result.no_changes_references.push(item.clone());
            }
        }
    }

    let ordered_result = build_ordered_result_list(&result);
    let result_names: HashSet<_> = ordered_result
        .iter()
        .map(|dependency| dependency.name.as_str())
        .collect();

    for dependency in dependencies {
        if !result_names.contains(dependency.name.as_str()) {
            result.deleted_references.push(dependency.clone());
        }
    }

    result
}

pub fn build_ordered_result_list(comparison_result: &DependencyComparisonResult) -> Vec<Dependency> {
    let mut ordered_dependency_list = Vec::new();
    ordered_dependency_list.extend(comparison_result.updated_references.clone());
    ordered_dependency_list.extend(comparison_result.no_changes_references.clone());
    ordered_dependency_list.extend(comparison_result.added_references.clone());

    let mut cross_list: Vec<_> = ordered_dependency_list
        .iter()
        .filter(|dependency| dependency.name.starts_with("Cross."))
        .cloned()
        .collect();
    cross_list.sort_by(|left, right| left.name.cmp(&right.name));
    ordered_dependency_list.retain(|dependency| !dependency.name.starts_with("Cross."));

    let mut boilerplate_list: Vec<_> = ordered_dependency_list
        .iter()
        .filter(|dependency| dependency.name.contains("Boilerplate"))
        .cloned()
        .collect();
    boilerplate_list.sort_by(|left, right| left.name.cmp(&right.name));
    ordered_dependency_list.retain(|dependency| !dependency.name.contains("Boilerplate"));

    let mut api_contract_list: Vec<_> = ordered_dependency_list
        .iter()
        .filter(|dependency| dependency.name.contains(".Api.Contract"))
        .cloned()
        .collect();
    api_contract_list.sort_by(|left, right| left.name.cmp(&right.name));
    ordered_dependency_list
        .retain(|dependency| !dependency.name.contains(".Api.Contract"));

    ordered_dependency_list.sort_by(|left, right| left.name.cmp(&right.name));

    let mut result_list = Vec::new();
    result_list.extend(cross_list);
    result_list.extend(boilerplate_list);
    result_list.extend(api_contract_list);
    result_list.extend(ordered_dependency_list);
    result_list
}

fn read_to_string(path: &Path) -> Result<String, LibError> {
    fs::read_to_string(path).map_err(|source| LibError::ReadFile {
        path: path.to_path_buf(),
        source,
    })
}

fn parse_xml(path: &Path, xml: &str) -> Result<Element, LibError> {
    xmltree::Element::parse(xml.as_bytes()).map_err(|error| LibError::InvalidXml {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

fn write_xml(path: &Path, root: &Element) -> Result<(), LibError> {
    let mut buffer = Vec::new();
    root.write_with_config(
        &mut buffer,
        xmltree::EmitterConfig::new()
            .perform_indent(true)
            .write_document_declaration(true)
            .pad_self_closing(true),
    )
    .map_err(|error| LibError::InvalidXml {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;

    fs::write(path, buffer).map_err(|source| LibError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

fn find_child_element<'a>(parent: &'a Element, name: &str) -> Option<&'a Element> {
    parent
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .find(|element| element.name == name)
}

fn find_child_element_mut<'a>(parent: &'a mut Element, name: &str) -> Option<&'a mut Element> {
    for child in &mut parent.children {
        if let XMLNode::Element(element) = child {
            if element.name == name {
                return Some(element);
            }
        }
    }

    None
}

fn element_text(element: &Element) -> Option<String> {
    element
        .children
        .iter()
        .filter_map(XMLNode::as_text)
        .next()
        .map(str::to_string)
}

fn dependency_from_element(element: &Element) -> Option<Dependency> {
    let id = element.attributes.get("id")?;
    let version = element.attributes.get("version")?;
    Some(Dependency::new(id.clone(), version.clone()))
}

fn get_group_dependencies(group: &Element) -> Vec<Dependency> {
    group
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .filter(|element| element.name == "dependency")
        .filter_map(dependency_from_element)
        .collect()
}

fn replace_flat_dependencies(root: &mut Element, result_list: &[Dependency]) {
    let metadata = find_child_element_mut(root, "metadata").expect("metadata");
    let dependencies_element = find_child_element_mut(metadata, "dependencies").expect("dependencies");
    let namespace = dependencies_element.namespace.clone();
    let prefix = dependencies_element.prefix.clone();
    let namespaces = dependencies_element.namespaces.clone();
    dependencies_element.children.clear();

    for dependency in result_list {
        dependencies_element.children.push(XMLNode::Element(Element {
            name: "dependency".to_string(),
            attributes: HashMap::from([
                ("id".to_string(), dependency.name.clone()),
                ("version".to_string(), dependency.version.clone()),
            ]),
            children: Vec::new(),
            namespace: namespace.clone(),
            prefix: prefix.clone(),
            namespaces: namespaces.clone(),
        }));
    }
}

fn apply_dependencies_to_single_group(
    root: &mut Element,
    target_framework: &str,
    result_list: &[Dependency],
) {
    let metadata = find_child_element_mut(root, "metadata").expect("metadata");
    let dependencies_element = find_child_element_mut(metadata, "dependencies").expect("dependencies");

    let group = dependencies_element
        .children
        .iter_mut()
        .filter_map(|child| match child {
            XMLNode::Element(element) => Some(element),
            _ => None,
        })
        .find(|element| {
            element.name == "group"
                && element
                    .attributes
                    .get("targetFramework")
                    .map(String::as_str)
                    .unwrap_or_default()
                    == target_framework
        })
        .expect("group");

    let result_by_name: HashMap<_, _> = result_list
        .iter()
        .map(|dependency| (dependency.name.as_str(), dependency.version.as_str()))
        .collect();

    group.children.retain_mut(|child| {
        let XMLNode::Element(dependency) = child else {
            return true;
        };

        if dependency.name != "dependency" {
            return true;
        }

        let Some(id) = dependency.attributes.get("id") else {
            return false;
        };

        match result_by_name.get(id.as_str()) {
            Some(version) => {
                dependency
                    .attributes
                    .insert("version".to_string(), (*version).to_string());
                true
            }
            None => false,
        }
    });

    let existing_ids: HashSet<String> = group
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .filter(|element| element.name == "dependency")
        .filter_map(|element| element.attributes.get("id").cloned())
        .collect();

    for added in result_list.iter().filter(|dependency| !existing_ids.contains(&dependency.name)) {
        group.children.push(XMLNode::Element(Element {
            name: "dependency".to_string(),
            attributes: HashMap::from([
                ("id".to_string(), added.name.clone()),
                ("version".to_string(), added.version.clone()),
            ]),
            children: Vec::new(),
            namespace: group.namespace.clone(),
            prefix: group.prefix.clone(),
            namespaces: group.namespaces.clone(),
        }));
    }
}

