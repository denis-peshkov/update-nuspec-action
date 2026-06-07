namespace UpdateNuspecTool.Tests;

[TestFixture]
public sealed class CsprojPackageReferenceResolverTests
{
    [Test]
    public void GetPackageReferencesForTargetFramework_resolves_property_versions_per_tfm()
    {
        var projectPath = Path.Combine(TestContext.CurrentContext.TestDirectory, "TestData", "Cross.Messaging.csproj");
        var project = XDocument.Load(projectPath);

        var net8 = CsprojPackageReferenceResolver.GetPackageReferencesForTargetFramework(project, "net8.0");
        net8.Should().Contain(p => p.Name == "MailKit" && p.Version == "4.16.0");
        net8.Should().Contain(p => p.Name == "Microsoft.Extensions.Configuration" && p.Version == "8.0.0");
        net8.Should().Contain(p => p.Name == "Microsoft.Extensions.Configuration.Binder" && p.Version == "8.0.2");

        var net9 = CsprojPackageReferenceResolver.GetPackageReferencesForTargetFramework(project, "net9.0");
        net9.Select(p => p.Name).Should().NotContain("Microsoft.Extensions.Configuration.Binder");
        net9.Should().Contain(p => p.Name == "Microsoft.Extensions.Configuration" && p.Version == "9.0.15");

        var net10 = CsprojPackageReferenceResolver.GetPackageReferencesForTargetFramework(project, "net10.0");
        net10.Should().Contain(p => p.Name == "Microsoft.Extensions.Configuration" && p.Version == "10.0.7");
    }

    [Test]
    public void GetPackageReferencesForTargetFramework_excludes_PrivateAssets_All()
    {
        var projectPath = Path.Combine(TestContext.CurrentContext.TestDirectory, "TestData", "Cross.Messaging.csproj");
        var project = XDocument.Load(projectPath);

        var packages = CsprojPackageReferenceResolver.GetPackageReferencesForTargetFramework(project, "net8.0");
        packages.Select(p => p.Name).Should().NotContain("Microsoft.SourceLink.GitHub");
    }

    [TestCase("'$(TargetFramework)' == 'net6.0'", "net6.0", true)]
    [TestCase("'$(TargetFramework)' == 'net6.0'", "net7.0", false)]
    [TestCase("$(TargetFramework) == 'net8.0'", "net8.0", true)]
    [TestCase("'$(TargetFramework)' == 'net6.0' or '$(TargetFramework)' == 'net7.0' or '$(TargetFramework)' == 'net8.0'", "net7.0", true)]
    [TestCase("'$(TargetFramework)' == 'net6.0' or '$(TargetFramework)' == 'net7.0' or '$(TargetFramework)' == 'net8.0'", "net9.0", false)]
    [TestCase(null, "net8.0", true)]
    [TestCase("'$(Configuration)' == 'Debug'", "net8.0", false)]
    [TestCase("'$(TargetFramework)' == 'net8.0'", "", false)]
    public void ConditionAppliesToTargetFramework_matches_standard_msbuild_condition(
        string? condition,
        string targetFramework,
        bool expected)
    {
        CsprojPackageReferenceResolver.ConditionAppliesToTargetFramework(condition, targetFramework)
            .Should()
            .Be(expected);
    }

    [Test]
    public void GetPackageReferences_uses_first_target_from_TargetFrameworks()
    {
        var project = XDocument.Parse(
            """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup>
                <TargetFrameworks>net8.0;net9.0</TargetFrameworks>
              </PropertyGroup>
              <ItemGroup>
                <PackageReference Include="Sample.Package" Version="2.0.0" />
              </ItemGroup>
            </Project>
            """);

        var packages = CsprojPackageReferenceResolver.GetPackageReferences(project);

        packages.Should().ContainSingle()
            .Which.Should().BeEquivalentTo(new Dependency("Sample.Package", "2.0.0"));
    }

    [Test]
    public void GetPackageReferences_uses_single_TargetFramework()
    {
        var project = XDocument.Parse(
            """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup>
                <TargetFramework>net8.0</TargetFramework>
              </PropertyGroup>
              <ItemGroup>
                <PackageReference Include="Single.Tfm" Version="1.2.3" />
              </ItemGroup>
            </Project>
            """);

        var packages = CsprojPackageReferenceResolver.GetPackageReferences(project);

        packages.Should().ContainSingle()
            .Which.Should().BeEquivalentTo(new Dependency("Single.Tfm", "1.2.3"));
    }

    [Test]
    public void GetPackageReferences_resolves_version_from_msbuild_property()
    {
        var project = XDocument.Parse(
            """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup>
                <TargetFramework>net8.0</TargetFramework>
                <MyPackageVersion>9.9.9</MyPackageVersion>
              </PropertyGroup>
              <ItemGroup>
                <PackageReference Include="Prop.Versioned" Version="$(MyPackageVersion)" />
              </ItemGroup>
            </Project>
            """);

        var packages = CsprojPackageReferenceResolver.GetPackageReferences(project);

        packages.Should().ContainSingle()
            .Which.Version.Should().Be("9.9.9");
    }

    [Test]
    public void GetPackageReferences_returns_empty_list_when_target_framework_is_unknown()
    {
        var project = XDocument.Parse(
            """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup />
            </Project>
            """);

        var packages = CsprojPackageReferenceResolver.GetPackageReferences(project);

        packages.Should().BeEmpty();
    }
}
