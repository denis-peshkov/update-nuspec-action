namespace UpdateNuspecTool.Tests;

/// <summary>
/// Temp-copy workspace for a single nuspec fixture (does not modify repo TestData).
/// </summary>
public abstract class NuspecFixtureTestBase
{
    protected abstract string NuspecFileName { get; }

    protected string CreateWorkspace() => TestWorkspace.CreateCopy();

    protected string GetNuspecPath(string workspace) => Path.Combine(workspace, NuspecFileName);

    protected void RunProcess(string workspace)
    {
        var nuspecPath = GetNuspecPath(workspace);
        var action = () => NuspecProcessorHelper.Process(nuspecPath, workspace, dryRun: false);
        action.Should().NotThrow();
    }
}
