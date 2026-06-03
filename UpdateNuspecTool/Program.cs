var path = args.AsQueryable().FirstOrDefault();
path = path ?? Directory.GetCurrentDirectory();

UpdateNuspec(path);

// Uncomment this for testing
//UpdateNuspec(Directory.GetParent(Environment.CurrentDirectory).Parent.Parent.FullName + @"\TestData");


static void UpdateNuspec(string path)
{
    var isPathValid = Path.Exists(path);
    if (isPathValid)
    {
        var nuspecFiles = Directory.EnumerateFiles(path, "*.nuspec").ToList();

        if (nuspecFiles.Any())
        {
            foreach (var nuspec in nuspecFiles)
            {
                ConsoleHelper.Write($"Start processing file: ", ConsoleColor.Gray);
                ConsoleHelper.Write($"{nuspec} \n", ConsoleColor.Cyan);
                NuspecProcessorHelper.Process(nuspec, path);
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
}
