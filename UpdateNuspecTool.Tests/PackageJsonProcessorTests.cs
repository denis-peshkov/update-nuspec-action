namespace UpdateNuspecTool.Tests;

[TestFixture]
public sealed class PackageJsonProcessorTests
{
    [Test]
    public void Process_updates_version_and_guru_dependencies()
    {
        var workspace = TestWorkspace.CreateCopy();
        var packageJsonPath = Path.Combine(workspace, "package.json");

        PackageJsonProcessorHelper.Process(packageJsonPath, "2.1.0", "@guru/", dryRun: false);

        var json = JsonNode.Parse(File.ReadAllText(packageJsonPath))!.AsObject();
        json["version"]!.GetValue<string>().Should().Be("2.1.0");
        json["dependencies"]!["@guru/core"]!.GetValue<string>().Should().Be("^2.1.0");
        json["dependencies"]!["lodash"]!.GetValue<string>().Should().Be("4.17.21");
        json["devDependencies"]!["@guru/dev-tools"]!.GetValue<string>().Should().Be("^2.1.0");
        json["peerDependencies"]!["@guru/shared"]!.GetValue<string>().Should().Be("^2.1.0");
        json["optionalDependencies"]!["@guru/optional"]!.GetValue<string>().Should().Be("^2.1.0");
    }

    [Test]
    public void Process_dry_run_does_not_modify_package_json()
    {
        var workspace = TestWorkspace.CreateCopy();
        var packageJsonPath = Path.Combine(workspace, "package.json");
        var before = File.ReadAllText(packageJsonPath);

        PackageJsonProcessorHelper.Process(packageJsonPath, "2.1.0", "@guru/", dryRun: true);

        File.ReadAllText(packageJsonPath).Should().Be(before);
    }

    [Test]
    public void Process_without_scope_updates_only_version()
    {
        var workspace = TestWorkspace.CreateCopy();
        var packageJsonPath = Path.Combine(workspace, "package.json");

        PackageJsonProcessorHelper.Process(packageJsonPath, "2.1.0", string.Empty, dryRun: false);

        var json = JsonNode.Parse(File.ReadAllText(packageJsonPath))!.AsObject();
        json["version"]!.GetValue<string>().Should().Be("2.1.0");
        json["dependencies"]!["@guru/core"]!.GetValue<string>().Should().Be("^1.0.0");
    }

    [Test]
    public void Main_with_package_version_updates_package_json()
    {
        var workspace = TestWorkspace.CreateCopy();
        var packageJsonPath = Path.Combine(workspace, "package.json");

        var (_, output) = ToolProcessRunner.Run(workspace, "--package-version", "3.0.0");

        output.Should().Contain("Start processing file");
        output.Should().Contain("package.json");
        JsonNode.Parse(File.ReadAllText(packageJsonPath))!["version"]!.GetValue<string>().Should().Be("3.0.0");
    }

    [Test]
    public void Main_skips_node_modules_package_json()
    {
        var workspace = Path.Combine(Path.GetTempPath(), "UpdateNuspecTool.Tests", Guid.NewGuid().ToString("N"));
        var nodeModulesDir = Path.Combine(workspace, "node_modules", "@guru", "pkg");
        Directory.CreateDirectory(nodeModulesDir);
        File.WriteAllText(
            Path.Combine(nodeModulesDir, "package.json"),
            """{"name":"nested","version":"9.9.9","dependencies":{"@guru/core":"^9.9.9"}}""");

        try
        {
            var (_, output) = ToolProcessRunner.Run(workspace, "--package-version", "1.0.0");

            output.Should().Contain("package.json files not found");
            File.ReadAllText(Path.Combine(nodeModulesDir, "package.json")).Should().Contain("9.9.9");
        }
        finally
        {
            Directory.Delete(workspace, recursive: true);
        }
    }
}
