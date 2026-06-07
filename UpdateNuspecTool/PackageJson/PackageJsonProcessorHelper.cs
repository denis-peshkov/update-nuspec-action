namespace UpdateNuspecTool.PackageJson;

public static class PackageJsonProcessorHelper
{
    private static readonly string[] DependencySections =
    [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ];

    private static readonly JsonSerializerOptions WriteOptions = new()
    {
        WriteIndented = true,
        Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping,
    };

    public static void Process(
        string filePath,
        string packageVersion,
        string? dependencyScopePrefix,
        bool dryRun)
    {
        try
        {
            var json = File.ReadAllText(filePath);
            var rootNode = JsonNode.Parse(json)?.AsObject();
            if (rootNode == null)
            {
                ConsoleHelper.WriteLine($"Invalid package.json: {filePath}", ConsoleColor.Red);
                return;
            }

            var oldVersion = rootNode["version"]?.GetValue<string>() ?? string.Empty;
            var versionChanged = !string.Equals(oldVersion, packageVersion, StringComparison.Ordinal);

            ConsoleHelper.Write("Current version: ", ConsoleColor.Gray);
            ConsoleHelper.WriteLine(oldVersion, ConsoleColor.Cyan);
            ConsoleHelper.Write("New version: ", ConsoleColor.Gray);
            ConsoleHelper.WriteLine(packageVersion, ConsoleColor.Cyan);

            var dependencyChanges = AlignScopedDependencies(rootNode, packageVersion, dependencyScopePrefix);

            if (versionChanged)
            {
                rootNode["version"] = packageVersion;
            }

            ShowResult(versionChanged, oldVersion, packageVersion, dependencyChanges);

            if (!dryRun)
            {
                var output = JsonSerializer.Serialize(rootNode, WriteOptions) + Environment.NewLine;
                File.WriteAllText(filePath, output);
            }
        }
        catch (Exception ex)
        {
            ConsoleHelper.WriteLine($"Error processing {filePath}: {ex.Message}", ConsoleColor.Red);
        }
    }

    private static List<PackageJsonChange> AlignScopedDependencies(
        JsonObject root,
        string packageVersion,
        string? dependencyScopePrefix)
    {
        var changes = new List<PackageJsonChange>();
        if (string.IsNullOrEmpty(dependencyScopePrefix))
        {
            return changes;
        }

        var newVersion = "^" + packageVersion;

        foreach (var section in DependencySections)
        {
            if (root[section] is not JsonObject sectionObject)
            {
                continue;
            }

            foreach (var property in sectionObject.ToList())
            {
                if (!property.Key.StartsWith(dependencyScopePrefix, StringComparison.Ordinal))
                {
                    continue;
                }

                var oldValue = property.Value?.GetValue<string>() ?? string.Empty;
                if (string.Equals(oldValue, newVersion, StringComparison.Ordinal))
                {
                    continue;
                }

                sectionObject[property.Key] = newVersion;
                changes.Add(new PackageJsonChange(section, property.Key, oldValue, newVersion));
            }
        }

        return changes;
    }

    private static void ShowResult(
        bool versionChanged,
        string oldVersion,
        string newVersion,
        IReadOnlyList<PackageJsonChange> dependencyChanges)
    {
        if (versionChanged)
        {
            ConsoleHelper.Write("\t Version: ", ConsoleColor.Gray);
            ConsoleHelper.WriteLine($"{oldVersion} -> {newVersion}", ConsoleColor.Yellow);
        }
        else
        {
            ConsoleHelper.WriteLine("\t Version: not changed", ConsoleColor.Gray);
        }

        if (!dependencyChanges.Any())
        {
            ConsoleHelper.WriteLine("\t Scoped dependencies: not changed", ConsoleColor.Gray);
            Console.WriteLine();
            return;
        }

        ConsoleHelper.WriteLine($"\t Updated scoped dependencies {dependencyChanges.Count}:", ConsoleColor.Yellow);
        foreach (var change in dependencyChanges.OrderBy(p => p.Section, StringComparer.Ordinal).ThenBy(p => p.Name, StringComparer.Ordinal))
        {
            ConsoleHelper.Write($"\t\t [{change.Section}] ", ConsoleColor.Gray);
            ConsoleHelper.Write($"{change.Name}: ", ConsoleColor.Yellow);
            ConsoleHelper.Write($"{change.OldVersion} ", ConsoleColor.Gray);
            ConsoleHelper.Write("-> ", ConsoleColor.Gray);
            ConsoleHelper.WriteLine(change.NewVersion, ConsoleColor.Yellow);
        }

        Console.WriteLine();
    }
}
