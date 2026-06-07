namespace UpdateNuspecTool.Nuspec;

public sealed class DependencyComparisonResult
{
    public List<Dependency> UpdatedReferences { get; } = [];
    public List<Dependency> AddedReferences { get; } = [];
    public List<Dependency> NoChangesReferences { get; } = [];
    public List<Dependency> DeletedReferences { get; } = [];
    public Dictionary<string, string> OutdatedReferences { get; } = new(StringComparer.Ordinal);
}
