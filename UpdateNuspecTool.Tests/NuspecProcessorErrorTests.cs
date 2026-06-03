namespace UpdateNuspecTool.Tests;

[TestFixture]
public sealed class NuspecProcessorErrorTests
{
    [Test]
    public void Process_missing_metadata_id_prints_error_and_returns()
    {
        var workspace = Path.Combine(Path.GetTempPath(), "UpdateNuspecTool.Tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(workspace);
        var nuspecPath = Path.Combine(workspace, "Broken.nuspec");
        File.WriteAllText(
            nuspecPath,
            """
            <?xml version="1.0" encoding="utf-8"?>
            <package xmlns="http://schemas.microsoft.com/packaging/2010/07/nuspec.xsd">
              <metadata>
                <version>1.0.0</version>
                <dependencies />
              </metadata>
            </package>
            """);

        using var capture = ConsoleCapture.Attach();

        NuspecProcessorHelper.Process(nuspecPath, workspace, dryRun: true);

        capture.Output.Should().Contain("ProjectName not found");
    }

    [Test]
    public void Process_missing_csproj_prints_error_and_returns()
    {
        var workspace = Path.Combine(Path.GetTempPath(), "UpdateNuspecTool.Tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(workspace);
        var nuspecPath = Path.Combine(workspace, "Orphan.nuspec");
        File.WriteAllText(
            nuspecPath,
            """
            <?xml version="1.0" encoding="utf-8"?>
            <package xmlns="http://schemas.microsoft.com/packaging/2010/07/nuspec.xsd">
              <metadata>
                <id>Orphan</id>
                <version>1.0.0</version>
                <dependencies />
              </metadata>
            </package>
            """);

        using var capture = ConsoleCapture.Attach();

        NuspecProcessorHelper.Process(nuspecPath, workspace, dryRun: true);

        capture.Output.Should().Contain("ProjectFile: Orphan not found");
    }

    [Test]
    public void Process_invalid_xml_rethrows()
    {
        var workspace = Path.Combine(Path.GetTempPath(), "UpdateNuspecTool.Tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(workspace);
        var nuspecPath = Path.Combine(workspace, "Bad.nuspec");
        File.WriteAllText(nuspecPath, "<not-valid-xml");

        var action = () => NuspecProcessorHelper.Process(nuspecPath, workspace, dryRun: true);

        action.Should().Throw<Exception>();
    }
}
