namespace UpdateNuspecTool.Tests;

internal static class ToolProcessRunner
{
    private static readonly string[] IsolatedEnvironmentVariables =
    [
        "PACKAGE_VERSION",
        "GITVERSION_SEMVER",
        "GitVersion_SemVer",
        "semVer",
        "SEMVER",
        "DEPENDENCY_SCOPE",
    ];

    public static (int ExitCode, string Output) Run(params string[] args)
    {
        var toolDll = typeof(CliHelper).Assembly.Location;
        var arguments = string.Join(' ', args.Select(Quote));
        arguments = string.IsNullOrEmpty(arguments) ? Quote(toolDll) : $"{Quote(toolDll)} {arguments}";

        using var process = new Process
        {
            StartInfo = new ProcessStartInfo
            {
                FileName = "dotnet",
                Arguments = arguments,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
            },
        };

        foreach (var name in IsolatedEnvironmentVariables)
        {
            process.StartInfo.Environment.Remove(name);
        }

        process.Start();
        var stdout = process.StandardOutput.ReadToEnd();
        var stderr = process.StandardError.ReadToEnd();
        process.WaitForExit();

        return (process.ExitCode, stdout + stderr);
    }

    private static string Quote(string value) => $"\"{value}\"";
}
