namespace UpdateNuspecTool.Tests;

internal sealed class ConsoleCapture : IDisposable
{
    private readonly TextWriter _originalOut;
    private readonly StringWriter _writer;

    private ConsoleCapture()
    {
        _originalOut = Console.Out;
        _writer = new StringWriter();
        Console.SetOut(_writer);
    }

    public string Output => _writer.ToString();

    public static ConsoleCapture Attach() => new();

    public void Dispose()
    {
        Console.SetOut(_originalOut);
        _writer.Dispose();
    }
}
