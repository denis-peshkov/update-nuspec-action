namespace UpdateNuspecTool.Tests;

internal static class TestWorkspace
{
    public static string TestDataDirectory =>
        Path.Combine(AppContext.BaseDirectory, "TestData");

    public static string CreateCopy()
    {
        var target = Path.Combine(Path.GetTempPath(), "UpdateNuspecTool.Tests", Guid.NewGuid().ToString("N"));
        CopyDirectory(TestDataDirectory, target);
        return target;
    }

    private static void CopyDirectory(string source, string target)
    {
        Directory.CreateDirectory(target);

        foreach (var file in Directory.EnumerateFiles(source, "*", SearchOption.AllDirectories))
        {
            var relative = Path.GetRelativePath(source, file);
            var destination = Path.Combine(target, relative);
            Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
            File.Copy(file, destination, overwrite: true);
        }
    }
}
