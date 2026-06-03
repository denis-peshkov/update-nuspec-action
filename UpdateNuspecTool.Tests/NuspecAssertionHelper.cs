namespace UpdateNuspecTool.Tests;

internal static class NuspecAssertionHelper
{
    public static IReadOnlyList<string> GetDependencyVersions(string nuspecPath, string packageId)
    {
        var document = XDocument.Load(nuspecPath);

        return document
            .Descendants()
            .Where(e => e.Name.LocalName == "dependency" && e.Attribute("id")?.Value == packageId)
            .Select(e => e.Attribute("version")!.Value)
            .ToList();
    }

    public static bool ContainsDependency(string nuspecPath, string packageId)
    {
        return GetDependencyVersions(nuspecPath, packageId).Any();
    }
}
