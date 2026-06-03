namespace UpdateNuspecTool;

internal static class CliHelper
{
    public static bool IsDryRunSwitch(string arg)
    {
        return arg.Equals("--dry-run", StringComparison.OrdinalIgnoreCase)
               || arg.Equals("-d", StringComparison.OrdinalIgnoreCase)
               || arg.Equals("--demo", StringComparison.OrdinalIgnoreCase)
               || arg.Equals("true", StringComparison.OrdinalIgnoreCase);
    }

    public static bool IsHelpSwitch(string arg)
    {
        return arg.Equals("--help", StringComparison.OrdinalIgnoreCase)
            || arg.Equals("-h", StringComparison.OrdinalIgnoreCase)
            || arg.Equals("-?", StringComparison.OrdinalIgnoreCase)
            || arg.Equals("/?", StringComparison.OrdinalIgnoreCase);
    }

    public static bool IsVersionSwitch(string arg)
    {
        return arg.Equals("--version", StringComparison.OrdinalIgnoreCase)
            || arg.Equals("-v", StringComparison.OrdinalIgnoreCase);
    }

    public static void PrintVersion()
    {
        var assembly = Assembly.GetExecutingAssembly();
        var informationalVersion = assembly
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()
            ?.InformationalVersion;
        var version = assembly.GetName().Version?.ToString(3);

        Console.WriteLine($"UpdateNuspecTool {informationalVersion ?? version ?? "unknown"}");
    }

    public static void PrintHelp()
    {
        var version = Assembly.GetExecutingAssembly().GetName().Version?.ToString(3) ?? "unknown";

        Console.WriteLine(
            $"""
             UpdateNuspecTool {version}
             Sync NuGet <dependencies> in *.nuspec with PackageReference versions from matching *.csproj files.

             DESCRIPTION
               Scans a directory for .nuspec files, finds a .csproj with the same name as <id> in nuspec metadata,
               compares package versions, and rewrites the <dependencies> section (flat or per targetFramework group).
               Use dry-run to preview changes without saving files.

             USAGE
               UpdateNuspecTool [path] [options]

             ARGUMENTS
               path                    Directory to scan (default: current directory).
                                       In GitHub Actions the action passes a path relative to /github/workspace.

             OPTIONS
               --help, -h, -?          Show this help.
               --version, -v           Show tool version.
               --dry-run, -d, --demo   Analyze and print the report; do not modify .nuspec files.
               true                    Same as --dry-run (positional boolean).

             EXAMPLES
               UpdateNuspecTool
               UpdateNuspecTool ./src/MyPackage
               UpdateNuspecTool ./TestData --dry-run
               UpdateNuspecTool -d .
               UpdateNuspecTool --version
               UpdateNuspecTool --help

             GITHUB ACTION
               - uses: denis-peshkov/update-nuspec-action@v1
                 with:
                   dir: src/MyPackage

             """);
    }
}
