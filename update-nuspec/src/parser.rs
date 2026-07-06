use std::collections::{HashMap, HashSet};

use regex::Regex;
use roxmltree::Document;

use crate::types::Dependency;

const METADATA_PROPERTY_NAMES: &[&str] = &[
    "TargetFramework",
    "TargetFrameworks",
    "TargetFrameworkIdentifier",
    "TargetFrameworkVersion",
    "LangVersion",
    "Nullable",
    "ImplicitUsings",
    "EnablePreviewFeatures",
    "Configurations",
    "Platforms",
    "NoWarn",
    "GenerateDocumentationFile",
    "DocumentationFile",
    "OutputType",
    "AssemblyName",
    "RootNamespace",
    "Version",
    "Authors",
    "Description",
];

pub fn get_package_references_for_target_framework(
    project_xml: &str,
    target_framework: &str,
) -> Result<Vec<Dependency>, String> {
    let document = Document::parse(project_xml).map_err(|error| error.to_string())?;
    let properties = build_properties_for_target_framework(&document, target_framework);
    Ok(collect_package_references(
        &document,
        target_framework,
        &properties,
    ))
}

pub fn get_package_references(project_xml: &str) -> Result<Vec<Dependency>, String> {
    let document = Document::parse(project_xml).map_err(|error| error.to_string())?;
    let target_framework = get_primary_target_framework(&document);
    let properties = build_properties_for_target_framework(&document, &target_framework);
    Ok(collect_package_references(
        &document,
        &target_framework,
        &properties,
    ))
}

pub fn condition_applies_to_target_framework(condition: Option<&str>, target_framework: &str) -> bool {
    let Some(condition) = condition.map(str::trim).filter(|value| !value.is_empty()) else {
        return true;
    };

    if target_framework.trim().is_empty() {
        return false;
    }

    static TARGET_FRAMEWORK_CONDITION_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let regex = TARGET_FRAMEWORK_CONDITION_REGEX.get_or_init(|| {
        Regex::new(r#"['"]?\$\(TargetFramework\)['"]?\s*==\s*['"]([^'"]+)['"]"#)
            .expect("valid target framework condition regex")
    });

    for captures in regex.captures_iter(condition) {
        if captures
            .get(1)
            .is_some_and(|value| value.as_str().eq_ignore_ascii_case(target_framework))
        {
            return true;
        }
    }

    false
}

fn get_primary_target_framework(document: &Document<'_>) -> String {
    let unconditional = document
        .descendants()
        .filter(|node| node.tag_name().name() == "PropertyGroup")
        .filter(|node| node.attribute("Condition").is_none());

    for property_group in unconditional {
        for child in property_group.children().filter(|node| node.is_element()) {
            if child.tag_name().name() == "TargetFramework" {
                let value = child.text().unwrap_or_default().trim();
                if !value.is_empty() {
                    return value.to_string();
                }
            }
        }
    }

    for property_group in document
        .descendants()
        .filter(|node| node.tag_name().name() == "PropertyGroup")
        .filter(|node| node.attribute("Condition").is_none())
    {
        for child in property_group.children().filter(|node| node.is_element()) {
            if child.tag_name().name() == "TargetFrameworks" {
                let value = child.text().unwrap_or_default().trim();
                if !value.is_empty() {
                    return value
                        .split(';')
                        .map(str::trim)
                        .find(|part| !part.is_empty())
                        .unwrap_or_default()
                        .to_string();
                }
            }
        }
    }

    String::new()
}

fn build_properties_for_target_framework(
    document: &Document<'_>,
    target_framework: &str,
) -> HashMap<String, String> {
    let metadata_names: HashSet<&str> = METADATA_PROPERTY_NAMES.iter().copied().collect();
    let mut properties = HashMap::new();

    for property_group in document
        .descendants()
        .filter(|node| node.tag_name().name() == "PropertyGroup")
    {
        if !condition_applies_to_target_framework(
            property_group.attribute("Condition"),
            target_framework,
        ) {
            continue;
        }

        for property in property_group.children().filter(|node| node.is_element()) {
            let local_name = property.tag_name().name();
            if metadata_names.contains(local_name) {
                continue;
            }

            properties.insert(
                local_name.to_string(),
                property.text().unwrap_or_default().trim().to_string(),
            );
        }
    }

    properties
}

fn collect_package_references(
    document: &Document<'_>,
    target_framework: &str,
    properties: &HashMap<String, String>,
) -> Vec<Dependency> {
    let mut packages: HashMap<String, Dependency> = HashMap::new();

    for item_group in document
        .descendants()
        .filter(|node| node.tag_name().name() == "ItemGroup")
    {
        if !condition_applies_to_target_framework(item_group.attribute("Condition"), target_framework)
        {
            continue;
        }

        for package_reference in item_group
            .children()
            .filter(|node| node.is_element())
            .filter(|node| node.tag_name().name() == "PackageReference")
        {
            if !condition_applies_to_target_framework(
                package_reference.attribute("Condition"),
                target_framework,
            ) {
                continue;
            }

            if should_exclude_from_nuspec(package_reference) {
                continue;
            }

            let Some(include) = package_reference
                .attribute("Include")
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };

            let version = resolve_version(package_reference, properties);
            packages.insert(
                include.to_ascii_lowercase(),
                Dependency::new(include, version),
            );
        }
    }

    packages.into_values().collect()
}

fn should_exclude_from_nuspec(package_reference: roxmltree::Node<'_, '_>) -> bool {
    let Some(private_assets) = package_reference.attribute("PrivateAssets") else {
        return false;
    };

    private_assets
        .split(';')
        .map(str::trim)
        .any(|part| part.eq_ignore_ascii_case("all"))
}

fn resolve_version(
    package_reference: roxmltree::Node<'_, '_>,
    properties: &HashMap<String, String>,
) -> String {
    let version = package_reference
        .attribute("Version")
        .or_else(|| {
            package_reference
                .children()
                .find(|node| node.is_element() && node.tag_name().name() == "Version")
                .and_then(|node| node.text())
        })
        .unwrap_or_default()
        .trim()
        .to_string();

    if version.starts_with("$(") && version.ends_with(')') {
        let property_name = &version[2..version.len() - 1];
        if let Some(resolved) = properties.get(property_name).filter(|value| !value.is_empty()) {
            return resolved.clone();
        }
    }

    version
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_applies_to_target_framework_matches_standard_msbuild_condition() {
        assert!(condition_applies_to_target_framework(
            Some("'$(TargetFramework)' == 'net6.0'"),
            "net6.0"
        ));
        assert!(!condition_applies_to_target_framework(
            Some("'$(TargetFramework)' == 'net6.0'"),
            "net7.0"
        ));
        assert!(condition_applies_to_target_framework(
            Some("$(TargetFramework) == 'net8.0'"),
            "net8.0"
        ));
        assert!(condition_applies_to_target_framework(
            Some(
                "'$(TargetFramework)' == 'net6.0' or '$(TargetFramework)' == 'net7.0' or '$(TargetFramework)' == 'net8.0'"
            ),
            "net7.0"
        ));
        assert!(!condition_applies_to_target_framework(
            Some(
                "'$(TargetFramework)' == 'net6.0' or '$(TargetFramework)' == 'net7.0' or '$(TargetFramework)' == 'net8.0'"
            ),
            "net9.0"
        ));
        assert!(condition_applies_to_target_framework(None, "net8.0"));
        assert!(!condition_applies_to_target_framework(
            Some("'$(Configuration)' == 'Debug'"),
            "net8.0"
        ));
        assert!(!condition_applies_to_target_framework(
            Some("'$(TargetFramework)' == 'net8.0'"),
            ""
        ));
    }

    #[test]
    fn get_package_references_uses_first_target_from_target_frameworks() {
        let project = r#"
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup>
                <TargetFrameworks>net8.0;net9.0</TargetFrameworks>
              </PropertyGroup>
              <ItemGroup>
                <PackageReference Include="Sample.Package" Version="2.0.0" />
              </ItemGroup>
            </Project>
        "#;

        let packages = get_package_references(project).expect("valid project");
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0], Dependency::new("Sample.Package", "2.0.0"));
    }

    #[test]
    fn get_package_references_resolves_version_from_msbuild_property() {
        let project = r#"
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup>
                <TargetFramework>net8.0</TargetFramework>
                <MyPackageVersion>9.9.9</MyPackageVersion>
              </PropertyGroup>
              <ItemGroup>
                <PackageReference Include="Prop.Versioned" Version="$(MyPackageVersion)" />
              </ItemGroup>
            </Project>
        "#;

        let packages = get_package_references(project).expect("valid project");
        assert_eq!(packages[0].version, "9.9.9");
    }
}
