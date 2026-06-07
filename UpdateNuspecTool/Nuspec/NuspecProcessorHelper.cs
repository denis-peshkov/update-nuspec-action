namespace UpdateNuspecTool.Nuspec;

public static class NuspecProcessorHelper
{
    public static void Process(string file, string path, bool dryRun = false)
    {
        try
        {
            var stopwatch = new Stopwatch();
            stopwatch.Start();

            var nuspecData = XDocument.Load(file);
            XNamespace nuspecDocNamespace = nuspecData.Root!.Name.Namespace;

            var projectName = nuspecData.Descendants(nuspecDocNamespace + "metadata")
                .Select(x => x.Element(nuspecDocNamespace + "id"))
                .First()
                ?.Value;

            if (projectName == null)
            {
                ConsoleHelper.WriteLine($"ProjectName not found in: {file}", ConsoleColor.Red);
                return;
            }

            var projectFilePath = Path.Combine(path, projectName + ".csproj");
            if (!File.Exists(projectFilePath))
            {
                ConsoleHelper.WriteLine($"ProjectFile: {projectName} not found in: {path}", ConsoleColor.Red);
                return;
            }

            ConsoleHelper.Write($"Processing project: ", ConsoleColor.Gray);
            ConsoleHelper.Write($"{projectFilePath} \n", ConsoleColor.Cyan);

            var projectData = XDocument.Load(projectFilePath);

            var dependenciesElement = nuspecData.Descendants(nuspecDocNamespace + "metadata")
                .Select(p => p.Element(nuspecDocNamespace + "dependencies"))
                .First()!;

            var dependencyGroups = dependenciesElement.Elements(nuspecDocNamespace + "group").ToList();

            if (dependencyGroups.Any())
            {
                foreach (var group in dependencyGroups)
                {
                    var targetFramework = group.Attribute("targetFramework")?.Value ?? string.Empty;
                    var groupDependencies = GetGroupDependencies(group, nuspecDocNamespace);
                    var packageReferences = CsprojPackageReferenceResolver
                        .GetPackageReferencesForTargetFramework(projectData, targetFramework);

                    var groupResult = CompareDependencies(groupDependencies, packageReferences);
                    ConsoleHelper.ShowGroupResult(
                        string.IsNullOrWhiteSpace(targetFramework) ? "(unknown)" : targetFramework,
                        groupResult);

                    if (!dryRun)
                    {
                        var resultList = BuildOrderedResultList(groupResult);
                        ApplyDependenciesToSingleGroup(group, resultList, nuspecDocNamespace);
                    }
                }
            }
            else
            {
                var dependencies = dependenciesElement.Elements(nuspecDocNamespace + "dependency")
                    .Where(p => p.Attribute("id") != null && p.Attribute("version") != null)
                    .Select(p => new Dependency(
                        p.Attribute("id")!.Value,
                        p.Attribute("version")!.Value))
                    .ToList();

                var packageReferences = CsprojPackageReferenceResolver.GetPackageReferences(projectData);
                var comparisonResult = CompareDependencies(dependencies, packageReferences);
                ConsoleHelper.ShowResult(comparisonResult);

                if (!dryRun)
                {
                    var resultList = BuildOrderedResultList(comparisonResult);
                    dependenciesElement.RemoveAll();
                    foreach (var value in resultList)
                    {
                        dependenciesElement.Add(new XElement(
                            nuspecDocNamespace + "dependency",
                            new XAttribute("id", value.Name),
                            new XAttribute("version", value.Version)));
                    }
                }
            }

            if (dryRun)
            {
                ConsoleHelper.WriteLine("[DRY RUN] Skipped saving nuspec file.", ConsoleColor.Yellow);
            }
            else
            {
                nuspecData.Save(file);
            }

            stopwatch.Stop();

            var ts = stopwatch.Elapsed;
            var elapsedTime = string.Format(
                "{0:00}:{1:00}:{2:00}.{3:00}",
                ts.Hours,
                ts.Minutes,
                ts.Seconds,
                ts.Milliseconds);

            Console.WriteLine($"Elapsed : {elapsedTime}");
        }
        catch (Exception)
        {
            throw;
        }
    }

    private static List<Dependency> GetGroupDependencies(XElement group, XNamespace nuspecDocNamespace)
    {
        return group.Elements(nuspecDocNamespace + "dependency")
            .Where(p => p.Attribute("id") != null && p.Attribute("version") != null)
            .Select(p => new Dependency(
                p.Attribute("id")!.Value,
                p.Attribute("version")!.Value))
            .ToList();
    }

    private static DependencyComparisonResult CompareDependencies(
        List<Dependency> dependencies,
        List<Dependency> packageReferences)
    {
        var result = new DependencyComparisonResult();

        foreach (var item in packageReferences)
        {
            var dependencyToUpdate = dependencies
                .Where(p => p.Name == item.Name)
                .ToList();

            if (!dependencyToUpdate.Any())
            {
                result.AddedReferences.Add(new Dependency(item.Name, item.Version));
                continue;
            }

            foreach (var dependency in dependencyToUpdate)
            {
                if (dependency.Version != item.Version)
                {
                    result.OutdatedReferences.TryAdd(dependency.Name, dependency.Version);
                    result.UpdatedReferences.Add(new Dependency(item.Name, item.Version));
                    continue;
                }

                result.NoChangesReferences.Add(new Dependency(item.Name, item.Version));
            }
        }

        var resultNames = BuildOrderedResultList(result).Select(p => p.Name).ToList();
        var dependencyNames = dependencies.Select(p => p.Name).ToList();
        var deletedNames = dependencyNames.Where(p => !resultNames.Contains(p)).ToList();
        result.DeletedReferences.AddRange(dependencies.Where(p => deletedNames.Contains(p.Name)));

        return result;
    }

    private static List<Dependency> BuildOrderedResultList(DependencyComparisonResult comparisonResult)
    {
        var orderedDependencyList = new List<Dependency>();
        orderedDependencyList.AddRange(comparisonResult.UpdatedReferences);
        orderedDependencyList.AddRange(comparisonResult.NoChangesReferences);
        orderedDependencyList.AddRange(comparisonResult.AddedReferences);

        var crossList = orderedDependencyList.Where(p => p.Name.StartsWith("Cross.")).OrderBy(p => p.Name).ToList();
        orderedDependencyList.RemoveAll(p => crossList.Contains(p));

        var boilerplateList = orderedDependencyList.Where(p => p.Name.Contains("Boilerplate")).OrderBy(p => p.Name).ToList();
        orderedDependencyList.RemoveAll(p => boilerplateList.Contains(p));

        var apiContractList = orderedDependencyList.Where(p => p.Name.Contains(".Api.Contract")).OrderBy(p => p.Name).ToList();
        orderedDependencyList.RemoveAll(p => apiContractList.Contains(p));

        orderedDependencyList = orderedDependencyList.OrderBy(p => p.Name).ToList();

        var resultList = new List<Dependency>();
        resultList.AddRange(crossList);
        resultList.AddRange(boilerplateList);
        resultList.AddRange(apiContractList);
        resultList.AddRange(orderedDependencyList);

        return resultList;
    }

    private static void ApplyDependenciesToSingleGroup(
        XElement group,
        List<Dependency> resultList,
        XNamespace nuspecDocNamespace)
    {
        var resultByName = resultList.ToDictionary(p => p.Name, p => p.Version);

        foreach (var dependency in group.Elements(nuspecDocNamespace + "dependency").ToList())
        {
            var id = dependency.Attribute("id")!.Value;
            if (resultByName.TryGetValue(id, out var version))
            {
                dependency.SetAttributeValue("version", version);
            }
            else
            {
                dependency.Remove();
            }
        }

        var existingIds = group.Elements(nuspecDocNamespace + "dependency")
            .Select(p => p.Attribute("id")!.Value)
            .ToHashSet();

        foreach (var added in resultList.Where(p => !existingIds.Contains(p.Name)))
        {
            group.Add(new XElement(
                nuspecDocNamespace + "dependency",
                new XAttribute("id", added.Name),
                new XAttribute("version", added.Version)));
        }
    }
}
