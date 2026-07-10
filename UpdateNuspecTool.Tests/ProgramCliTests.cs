namespace UpdateNuspecTool.Tests;

[TestFixture]
public sealed class ProgramCliTests
{
    [Test]
    public void Main_with_help_prints_usage()
    {
        var (_, output) = ToolProcessRunner.Run("--help");

        output.Should().Contain("USAGE");
        output.Should().Contain("UpdateNuspecTool");
    }

    [Test]
    public void Main_with_version_prints_version_line()
    {
        var (_, output) = ToolProcessRunner.Run("--version");

        output.Trim().Should().Be(CliHelper.GetVersion());
    }

    [Test]
    public void Main_with_invalid_path_prints_error()
    {
        var path = Path.Combine(Path.GetTempPath(), "UpdateNuspecTool.Tests", Guid.NewGuid().ToString("N"), "missing");
        var (_, output) = ToolProcessRunner.Run(path);

        output.Should().Contain("is not valid");
    }

    [Test]
    public void Main_with_empty_directory_prints_nuspec_not_found()
    {
        var workspace = Path.Combine(Path.GetTempPath(), "UpdateNuspecTool.Tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(workspace);

        try
        {
            var (_, output) = ToolProcessRunner.Run(workspace);

            output.Should().Contain("*.nuspec files not found");
        }
        finally
        {
            Directory.Delete(workspace, recursive: true);
        }
    }

    [Test]
    public void Main_with_dry_run_does_not_modify_nuspec()
    {
        var workspace = TestWorkspace.CreateCopy();
        var nuspecPath = Path.Combine(workspace, "MyPackage.nuspec");
        var before = File.ReadAllText(nuspecPath);

        var (_, output) = ToolProcessRunner.Run(workspace, "--dry-run");

        output.Should().Contain("[DRY RUN]");
        output.Should().Contain("Start processing file");
        File.ReadAllText(nuspecPath).Should().Be(before);
    }

    [Test]
    public void Main_processes_nuspec_in_workspace()
    {
        var workspace = TestWorkspace.CreateCopy();

        var (_, output) = ToolProcessRunner.Run(workspace);

        output.Should().Contain("Start processing file");
        NuspecAssertionHelper
            .GetDependencyVersions(Path.Combine(workspace, "MyPackage.nuspec"), "Newtonsoft.Json")
            .Should()
            .AllBeEquivalentTo("13.0.3");
    }
}
