namespace UpdateNuspecTool;

internal static partial class CsprojPackageReferenceResolver
{
    private static readonly HashSet<string> MetadataPropertyNames = new(StringComparer.OrdinalIgnoreCase)
    {
        "TargetFramework",
        "TargetFrameworks",
        "TargetFrameworkIdentifier",
        "TargetFrameworkVersion",
        "LangVersion",
        "Nullable",
        "ImplicitUsings",
        "EnablePreviewFeatures",
        "Configurations",
        "Platforms",
        "NoWarn",
        "GenerateDocumentationFile",
        "DocumentationFile",
        "OutputType",
        "AssemblyName",
        "RootNamespace",
        "Version",
        "Authors",
        "Description",
    };

    public static List<Dependency> GetPackageReferencesForTargetFramework(XDocument project, string targetFramework)
    {
        var properties = BuildPropertiesForTargetFramework(project, targetFramework);
        return CollectPackageReferences(project, targetFramework, properties);
    }

    public static List<Dependency> GetPackageReferences(XDocument project)
    {
        var targetFramework = GetPrimaryTargetFramework(project);
        return GetPackageReferencesForTargetFramework(project, targetFramework);
    }

    private static string GetPrimaryTargetFramework(XDocument project)
    {
        var unconditional = project.Descendants("PropertyGroup")
            .Where(pg => pg.Attribute("Condition") == null)
            .SelectMany(pg => pg.Elements())
            .ToList();

        var single = unconditional.FirstOrDefault(e => e.Name.LocalName == "TargetFramework")?.Value;
        if (!string.IsNullOrWhiteSpace(single))
        {
            return single.Trim();
        }

        var plural = unconditional.FirstOrDefault(e => e.Name.LocalName == "TargetFrameworks")?.Value;
        if (!string.IsNullOrWhiteSpace(plural))
        {
            return plural.Split(';', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)[0];
        }

        return string.Empty;
    }

    private static Dictionary<string, string> BuildPropertiesForTargetFramework(
        XDocument project,
        string targetFramework)
    {
        var properties = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);

        foreach (var propertyGroup in project.Descendants("PropertyGroup"))
        {
            if (!ConditionAppliesToTargetFramework(propertyGroup.Attribute("Condition")?.Value, targetFramework))
            {
                continue;
            }

            foreach (var property in propertyGroup.Elements())
            {
                if (MetadataPropertyNames.Contains(property.Name.LocalName))
                {
                    continue;
                }

                properties[property.Name.LocalName] = property.Value.Trim();
            }
        }

        return properties;
    }

    private static List<Dependency> CollectPackageReferences(
        XDocument project,
        string targetFramework,
        IReadOnlyDictionary<string, string> properties)
    {
        var packages = new Dictionary<string, Dependency>(StringComparer.OrdinalIgnoreCase);

        foreach (var itemGroup in project.Descendants("ItemGroup"))
        {
            if (!ConditionAppliesToTargetFramework(itemGroup.Attribute("Condition")?.Value, targetFramework))
            {
                continue;
            }

            foreach (var packageReference in itemGroup.Elements("PackageReference"))
            {
                if (!ConditionAppliesToTargetFramework(packageReference.Attribute("Condition")?.Value, targetFramework))
                {
                    continue;
                }

                if (ShouldExcludeFromNuspec(packageReference))
                {
                    continue;
                }

                var include = packageReference.Attribute("Include")?.Value;
                if (string.IsNullOrWhiteSpace(include))
                {
                    continue;
                }

                var version = ResolveVersion(packageReference, properties);
                packages[include] = new Dependency(include, version);
            }
        }

        return packages.Values.ToList();
    }

    private static bool ShouldExcludeFromNuspec(XElement packageReference)
    {
        var privateAssets = packageReference.Attribute("PrivateAssets")?.Value;
        if (string.IsNullOrWhiteSpace(privateAssets))
        {
            return false;
        }

        return privateAssets.Split(';', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
            .Any(part => part.Equals("all", StringComparison.OrdinalIgnoreCase));
    }

    private static string ResolveVersion(XElement packageReference, IReadOnlyDictionary<string, string> properties)
    {
        var version = packageReference.Attribute("Version")?.Value
            ?? packageReference.Element("Version")?.Value
            ?? string.Empty;

        version = version.Trim();
        if (version.StartsWith("$(", StringComparison.Ordinal) && version.EndsWith(')'))
        {
            var propertyName = version[2..^1];
            if (properties.TryGetValue(propertyName, out var resolved) && !string.IsNullOrWhiteSpace(resolved))
            {
                return resolved;
            }
        }

        return version;
    }

    internal static bool ConditionAppliesToTargetFramework(string? condition, string targetFramework)
    {
        if (string.IsNullOrWhiteSpace(condition))
        {
            return true;
        }

        if (string.IsNullOrWhiteSpace(targetFramework))
        {
            return false;
        }

        var matches = TargetFrameworkConditionRegex().Matches(condition);
        if (matches.Count == 0)
        {
            return false;
        }

        foreach (Match match in matches)
        {
            if (string.Equals(match.Groups[1].Value, targetFramework, StringComparison.OrdinalIgnoreCase))
            {
                return true;
            }
        }

        return false;
    }

    [GeneratedRegex(@"['""]?\$\(TargetFramework\)['""]?\s*==\s*['""]([^'""]+)['""]", RegexOptions.IgnoreCase | RegexOptions.CultureInvariant)]
    private static partial Regex TargetFrameworkConditionRegex();
}
