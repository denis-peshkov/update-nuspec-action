namespace UpdateNuspecTool.Tests;

[TestFixture]
public sealed class CliHelperTests
{
    [Test]
    public void PrintHelp_writes_usage()
    {
        using var capture = ConsoleCapture.Attach();

        CliHelper.PrintHelp();

        capture.Output.Should().Contain("UpdateNuspecTool");
        capture.Output.Should().Contain("USAGE");
    }

    [Test]
    public void PrintVersion_writes_version_line()
    {
        using var capture = ConsoleCapture.Attach();

        CliHelper.PrintVersion();

        capture.Output.Should().Contain("UpdateNuspecTool");
    }
}
