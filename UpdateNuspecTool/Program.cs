if (args.Any(CliHelper.IsHelpSwitch))
{
    CliHelper.PrintHelp();
    return;
}

if (args.Any(CliHelper.IsVersionSwitch))
{
    CliHelper.PrintVersion();
    return;
}

var dryRun = false;
string? path = null;

foreach (var arg in args)
{
    if (CliHelper.IsHelpSwitch(arg) || CliHelper.IsVersionSwitch(arg))
    {
        continue;
    }

    if (CliHelper.IsDryRunSwitch(arg))
    {
        dryRun = true;
        continue;
    }

    if (path == null)
    {
        path = arg;
    }
}

path ??= Directory.GetCurrentDirectory();

UpdateNuspec(path, dryRun);

static void UpdateNuspec(string path, bool dryRun)
{
    if (dryRun)
    {
        ConsoleHelper.WriteLine("[DRY RUN] Files will not be modified.", ConsoleColor.Yellow);
    }

    var isPathValid = Path.Exists(path);
    if (isPathValid)
    {
        var nuspecFiles = Directory
            .EnumerateFiles(path, "*.nuspec", SearchOption.AllDirectories)
            .ToList();

        if (nuspecFiles.Any())
        {
            foreach (var nuspec in nuspecFiles)
            {
                var nuspecDirectory = Path.GetDirectoryName(nuspec)!;
                ConsoleHelper.Write($"Start processing file: ", ConsoleColor.Gray);
                ConsoleHelper.Write($"{nuspec} \n", ConsoleColor.Cyan);
                NuspecProcessorHelper.Process(nuspec, nuspecDirectory, dryRun);
                ConsoleHelper.Write($"End processing file: ", ConsoleColor.Gray);
                ConsoleHelper.Write($"{nuspec} \n \n", ConsoleColor.Cyan);
            }
        }
        else
        {
            ConsoleHelper.WriteLine("*.nuspec files not found!", ConsoleColor.Red);
        }
    }
    else
    {
        ConsoleHelper.WriteLine($"Path '{path}' is not valid!", ConsoleColor.Red);
    }

    if (dryRun)
    {
        ConsoleHelper.WriteLine("[DRY RUN] Completed without writing changes.", ConsoleColor.Yellow);
    }
}
