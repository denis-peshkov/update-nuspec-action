namespace UpdateNuspecTool.Tests;

[TestFixture]
public sealed class CliParseTests
{
    [Test]
    public void ParseArgs_reads_package_version_and_scope()
    {
        var options = CliHelper.ParseArgs(["./dist", "--package-version", "1.2.3", "--dependency-scope", "@guru/", "--dry-run"]);

        options.Path.Should().Be("./dist");
        options.PackageVersion.Should().Be("1.2.3");
        options.DependencyScope.Should().Be("@guru/");
        options.DryRun.Should().BeTrue();
    }

    [Test]
    public void ParseArgs_reads_inline_option_values()
    {
        var options = CliHelper.ParseArgs(["./dist", "-pv=2.0.0", "-ds="]);

        options.PackageVersion.Should().Be("2.0.0");
        options.DependencyScope.Should().BeEmpty();
    }

    [Test]
    public void ResolvePackageVersion_prefers_cli_over_env()
    {
        Environment.SetEnvironmentVariable("PACKAGE_VERSION", "9.9.9");

        try
        {
            CliHelper.ResolvePackageVersion("1.0.0").Should().Be("1.0.0");
        }
        finally
        {
            Environment.SetEnvironmentVariable("PACKAGE_VERSION", null);
        }
    }

    [Test]
    public void ResolvePackageVersion_reads_gitversion_env()
    {
        Environment.SetEnvironmentVariable("GitVersion_SemVer", "1.3.0-preview.4");

        try
        {
            CliHelper.ResolvePackageVersion(null).Should().Be("1.3.0-preview.4");
        }
        finally
        {
            Environment.SetEnvironmentVariable("GitVersion_SemVer", null);
        }
    }

    [Test]
    public void ResolveDependencyScope_uses_empty_default_when_not_provided()
    {
        Environment.SetEnvironmentVariable("DEPENDENCY_SCOPE", null);

        CliHelper.ResolveDependencyScope(null, cliProvided: false).Should().BeEmpty();
    }
}
