namespace UpdateNuspecTool.Helper;

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

    public static CliRunOptions ParseArgs(string[] args)
    {
        string? path = null;
        var dryRun = false;
        var showHelp = false;
        var showVersion = false;
        string? packageVersion = null;
        string? dependencyScope = null;
        var dependencyScopeProvided = false;

        for (var i = 0; i < args.Length; i++)
        {
            var arg = args[i];

            if (IsHelpSwitch(arg))
            {
                showHelp = true;
                continue;
            }

            if (IsVersionSwitch(arg))
            {
                showVersion = true;
                continue;
            }

            if (IsDryRunSwitch(arg))
            {
                dryRun = true;
                continue;
            }

            if (IsOption(arg, "--package-version", "-pv"))
            {
                packageVersion = ReadOptionValue(args, ref i, arg);
                continue;
            }

            if (IsOption(arg, "--dependency-scope", "-ds"))
            {
                dependencyScope = ReadOptionValue(args, ref i, arg);
                dependencyScopeProvided = true;
                continue;
            }

            if (path == null && !arg.StartsWith('-'))
            {
                path = arg;
            }
        }

        return new CliRunOptions
        {
            Path = path ?? Directory.GetCurrentDirectory(),
            DryRun = dryRun,
            ShowHelp = showHelp,
            ShowVersion = showVersion,
            PackageVersion = ResolvePackageVersion(packageVersion),
            DependencyScope = ResolveDependencyScope(dependencyScope, dependencyScopeProvided),
        };
    }

    public static string? ResolvePackageVersion(string? cliValue)
    {
        if (!string.IsNullOrWhiteSpace(cliValue))
        {
            return cliValue.Trim();
        }

        foreach (var name in new[] { "PACKAGE_VERSION", "GITVERSION_SEMVER", "GitVersion_SemVer", "semVer", "SEMVER" })
        {
            var value = Environment.GetEnvironmentVariable(name);
            if (!string.IsNullOrWhiteSpace(value))
            {
                return value.Trim();
            }
        }

        return null;
    }

    public static string? ResolveDependencyScope(string? cliValue, bool cliProvided)
    {
        if (cliProvided)
        {
            return cliValue ?? string.Empty;
        }

        var envValue = Environment.GetEnvironmentVariable("DEPENDENCY_SCOPE");
        if (envValue != null)
        {
            return envValue;
        }

        return string.Empty;
    }

    public static string GetVersion()
    {
        var assembly = Assembly.GetExecutingAssembly();
        var informationalVersion = assembly
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()
            ?.InformationalVersion;
        var version = assembly.GetName().Version?.ToString(3);

        return $"{informationalVersion ?? version ?? "unknown"}";
    }

    public static void PrintVersion()
        => Console.WriteLine($"UpdateNuspecTool {GetVersion()}");

    public static string GetHelpText()
    {
        return
            $"""
             UpdateNuspecTool {GetVersion()}
             Sync NuGet <dependencies> in *.nuspec with PackageReference versions from matching *.csproj files.
             Optionally update package.json version and scoped npm dependencies.

             DESCRIPTION
               Recursively scans a directory for .nuspec files, finds a .csproj with the same name as <id>
               in nuspec metadata in each file's folder, compares package versions, and rewrites the
               <dependencies> section (flat or per targetFramework group).
               When --package-version (or PACKAGE_VERSION / GitVersion_SemVer / SemVer env) is set, also updates
               package.json: sets version as x.y.z and aligns dependencies whose names start with the scope prefix
               to ^x.y.z in dependencies, devDependencies, peerDependencies, optionalDependencies when --dependency-scope is set.
               Use dry-run to preview changes without saving files.

             USAGE
               UpdateNuspecTool [path] [options]

             ARGUMENTS
               path                    Root directory to scan recursively (default: current directory).
                                       In GitHub Actions the action passes a path relative to /github/workspace.

             OPTIONS
               --help, -h, -?          Show this help.
               --version, -v           Show tool version.
               --dry-run, -d, --demo   Analyze and print the report; do not modify files.
               true                    Same as --dry-run (positional boolean).
               --package-version, -pv  SemVer for package.json "version" (env: PACKAGE_VERSION, GitVersion_SemVer, semVer).
               --dependency-scope, -ds Scope prefix for npm dependency alignment (env: DEPENDENCY_SCOPE).
                                       Skipped when empty.

             EXAMPLES
               UpdateNuspecTool
               UpdateNuspecTool ./src/MyPackage
               UpdateNuspecTool ./client/dist/my-app --package-version 1.2.3
               UpdateNuspecTool ./client/dist/my-app -pv 1.2.3-preview.4 -ds @guru/
               UpdateNuspecTool ./UpdateNuspecTool.Tests/TestData --dry-run
               UpdateNuspecTool -d .
               UpdateNuspecTool --version
               UpdateNuspecTool --help

             GITHUB ACTION
               - uses: denis-peshkov/update-nuspec-action@v1
                 with:
                   dir: client/dist/my-app
                   packageVersion: 1.2.3

             """;
    }

    public static void PrintHelp()
        => Console.WriteLine(GetHelpText());

    public static void Run(CliRunOptions options)
    {
        ConsoleHelper.DryRun = options.DryRun;

        if (options.DryRun)
        {
            ConsoleHelper.WriteLine("[DRY RUN] Files will not be modified.", ConsoleColor.Yellow);
        }

        var isPathValid = Path.Exists(options.Path);
        if (!isPathValid)
        {
            ConsoleHelper.WriteLine($"Path '{options.Path}' is not valid!", ConsoleColor.Red);
            return;
        }

        UpdateNuspecFiles(options.Path, options.DryRun, suppressNotFoundMessage: !string.IsNullOrWhiteSpace(options.PackageVersion));

        if (!string.IsNullOrWhiteSpace(options.PackageVersion))
        {
            UpdatePackageJsonFiles(options.Path, options.PackageVersion, options.DependencyScope, options.DryRun);
        }

        if (options.DryRun)
        {
            ConsoleHelper.WriteLine("[DRY RUN] Completed without writing changes.", ConsoleColor.Yellow);
        }
    }

    private static void UpdateNuspecFiles(string path, bool dryRun, bool suppressNotFoundMessage = false)
    {
        var nuspecFiles = Directory
            .EnumerateFiles(path, "*.nuspec", SearchOption.AllDirectories)
            .ToList();

        if (nuspecFiles.Any())
        {
            foreach (var nuspec in nuspecFiles)
            {
                var nuspecDirectory = Path.GetDirectoryName(nuspec)!;
                ConsoleHelper.Write("Start processing file: ", ConsoleColor.Gray);
                ConsoleHelper.Write($"{nuspec} \n", ConsoleColor.Cyan);
                NuspecProcessorHelper.Process(nuspec, nuspecDirectory, dryRun);
                ConsoleHelper.Write("End processing file: ", ConsoleColor.Gray);
                ConsoleHelper.Write($"{nuspec} \n \n", ConsoleColor.Cyan);
            }
        }
        else if (!suppressNotFoundMessage)
        {
            ConsoleHelper.WriteLine("*.nuspec files not found!", ConsoleColor.Red);
        }
    }

    private static void UpdatePackageJsonFiles(string path, string packageVersion, string? dependencyScope, bool dryRun)
    {
        var packageJsonFiles = Directory
            .EnumerateFiles(path, "package.json", SearchOption.AllDirectories)
            .Where(p => !IsUnderNodeModules(p))
            .ToList();

        if (!packageJsonFiles.Any())
        {
            ConsoleHelper.WriteLine("package.json files not found!", ConsoleColor.Red);
            return;
        }

        foreach (var packageJson in packageJsonFiles)
        {
            ConsoleHelper.Write("Start processing file: ", ConsoleColor.Gray);
            ConsoleHelper.Write($"{packageJson} \n", ConsoleColor.Cyan);
            PackageJsonProcessorHelper.Process(packageJson, packageVersion, dependencyScope, dryRun);
            ConsoleHelper.Write("End processing file: ", ConsoleColor.Gray);
            ConsoleHelper.Write($"{packageJson} \n \n", ConsoleColor.Cyan);
        }
    }

    private static bool IsUnderNodeModules(string filePath)
    {
        var parts = filePath.Split(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
        return parts.Any(part => part.Equals("node_modules", StringComparison.OrdinalIgnoreCase));
    }

    private static bool IsOption(string arg, string longName, string shortName)
    {
        return arg.Equals(longName, StringComparison.OrdinalIgnoreCase)
            || arg.Equals(shortName, StringComparison.OrdinalIgnoreCase)
            || arg.StartsWith(longName + "=", StringComparison.OrdinalIgnoreCase)
            || arg.StartsWith(shortName + "=", StringComparison.OrdinalIgnoreCase);
    }

    private static string ReadOptionValue(string[] args, ref int index, string arg)
    {
        var equalIndex = arg.IndexOf('=');
        if (equalIndex >= 0)
        {
            return arg[(equalIndex + 1)..];
        }

        if (index + 1 >= args.Length)
        {
            throw new ArgumentException($"Missing value for option '{arg}'.");
        }

        index++;
        return args[index];
    }
}
