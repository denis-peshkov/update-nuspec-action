namespace UpdateNuspecTool.Tests;

[TestFixture]
public sealed class ConsoleHelperTests
{
    [Test]
    public void ShowResult_with_no_dependencies_prints_placeholder()
    {
        using var capture = ConsoleCapture.Attach();

        ConsoleHelper.ShowResult(
            [],
            [],
            [],
            [],
            new Dictionary<string, string>());

        capture.Output.Should().Contain("(no dependency changes)");
    }
}
