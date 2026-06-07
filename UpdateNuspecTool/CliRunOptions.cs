namespace UpdateNuspecTool;

internal sealed class CliRunOptions
{
    public string Path { get; init; } = Directory.GetCurrentDirectory();

    public bool DryRun { get; init; }

    public bool ShowHelp { get; init; }

    public bool ShowVersion { get; init; }

    public string? PackageVersion { get; init; }

    public string? DependencyScope { get; init; }
}
