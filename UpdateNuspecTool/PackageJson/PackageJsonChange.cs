namespace UpdateNuspecTool.PackageJson;

internal sealed record PackageJsonChange(string Section, string Name, string OldVersion, string NewVersion);
