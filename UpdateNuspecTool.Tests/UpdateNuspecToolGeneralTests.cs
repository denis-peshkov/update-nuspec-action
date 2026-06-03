namespace UpdateNuspecTool.Tests;

[TestFixture]
public sealed class UpdateNuspecToolGeneralTests
{
    private static readonly string[] KnownNuspecFiles =
    [
        "cgf.nuspec",
        "config.nuspec",
        "Cross.Messaging.nuspec",
        "MyPackage.nuspec",
    ];

    [Test]
    public void TestData_is_copied_to_output()
    {
        Directory.Exists(TestWorkspace.TestDataDirectory).Should().BeTrue();
        Directory.EnumerateFiles(TestWorkspace.TestDataDirectory, "*.nuspec").Should().NotBeEmpty();
    }

    [Test]
    public void Every_nuspec_in_TestData_has_dedicated_process_test_fixture()
    {
        var nuspecFiles = Directory
            .EnumerateFiles(TestWorkspace.TestDataDirectory, "*.nuspec")
            .Select(Path.GetFileName)
            .OrderBy(p => p)
            .ToList();

        nuspecFiles.Should().BeEquivalentTo(KnownNuspecFiles);
    }

    [TestCase("MyPackage.nuspec")]
    [TestCase("Cross.Messaging.nuspec")]
    [TestCase("config.nuspec")]
    [TestCase("cgf.nuspec")]
    public void DryRun_does_not_modify_nuspec(string nuspecFileName)
    {
        var workspace = TestWorkspace.CreateCopy();
        var nuspecPath = Path.Combine(workspace, nuspecFileName);
        var before = File.ReadAllText(nuspecPath);

        var action = () => NuspecProcessorHelper.Process(nuspecPath, workspace, dryRun: true);

        action.Should().NotThrow();
        File.ReadAllText(nuspecPath).Should().Be(before);
    }

    [TestCase("--dry-run", true)]
    [TestCase("-d", true)]
    [TestCase("--demo", true)]
    [TestCase("true", true)]
    [TestCase("--help", false)]
    [TestCase("-v", false)]
    public void IsDryRunSwitch_recognizes_flags(string arg, bool expected)
    {
        CliHelper.IsDryRunSwitch(arg).Should().Be(expected);
    }

    [TestCase("--help", true)]
    [TestCase("-h", true)]
    [TestCase("-?", true)]
    [TestCase("/?", true)]
    [TestCase("--version", false)]
    public void IsHelpSwitch_recognizes_flags(string arg, bool expected)
    {
        CliHelper.IsHelpSwitch(arg).Should().Be(expected);
    }

    [TestCase("--version", true)]
    [TestCase("-v", true)]
    [TestCase("--help", false)]
    public void IsVersionSwitch_recognizes_flags(string arg, bool expected)
    {
        CliHelper.IsVersionSwitch(arg).Should().Be(expected);
    }

    [Test]
    public void GetHelpText_contains_usage_and_description()
    {
        var text = CliHelper.GetHelpText();

        text.Should().Contain("UpdateNuspecTool");
        text.Should().Contain("DESCRIPTION");
        text.Should().Contain("USAGE");
        text.Should().Contain("--dry-run");
        text.Should().Contain("GITHUB ACTION");
    }

    [Test]
    public void GetVersionText_matches_assembly_version()
    {
        var assembly = typeof(CliHelper).Assembly;
        var expected = assembly
            .GetCustomAttribute<AssemblyInformationalVersionAttribute>()
            ?.InformationalVersion
            ?? assembly.GetName().Version?.ToString(3)
            ?? "unknown";

        CliHelper.GetVersionText().Should().Be($"UpdateNuspecTool {expected}");
    }
}
