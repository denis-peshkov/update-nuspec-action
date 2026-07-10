use update_nuspec::{condition_applies_to_target_framework, get_package_references, Dependency};

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

#[test]
fn get_package_references_excludes_private_assets_all() {
    let project = r#"
        <Project Sdk="Microsoft.NET.Sdk">
          <PropertyGroup>
            <TargetFramework>net8.0</TargetFramework>
          </PropertyGroup>
          <ItemGroup>
            <PackageReference Include="Visible.Package" Version="1.0.0" />
            <PackageReference Include="Hidden.Package" Version="2.0.0" PrivateAssets="All" />
          </ItemGroup>
        </Project>
    "#;

    let packages = get_package_references(project).expect("valid project");
    let names: Vec<_> = packages.iter().map(|dependency| dependency.name.as_str()).collect();

    assert_eq!(names, vec!["Visible.Package"]);
}

#[test]
fn get_package_references_reads_version_from_child_element() {
    let project = r#"
        <Project Sdk="Microsoft.NET.Sdk">
          <PropertyGroup>
            <TargetFramework>net8.0</TargetFramework>
          </PropertyGroup>
          <ItemGroup>
            <PackageReference Include="Child.Versioned">
              <Version>3.3.3</Version>
            </PackageReference>
          </ItemGroup>
        </Project>
    "#;

    let packages = get_package_references(project).expect("valid project");
    assert_eq!(packages[0], Dependency::new("Child.Versioned", "3.3.3"));
}

#[test]
fn get_package_references_returns_error_for_invalid_xml() {
    let error = get_package_references("<Project").expect_err("invalid xml");
    assert!(!error.is_empty());
}
