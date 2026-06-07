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

    [Test]
    public void ShowResult_with_all_change_types_prints_sections()
    {
        using var capture = ConsoleCapture.Attach();

        ConsoleHelper.ShowResult(
            [new Dependency("Updated.Package", "2.0.0")],
            [new Dependency("Same.Package", "1.0.0")],
            [new Dependency("Added.Package", "3.0.0")],
            [new Dependency("Removed.Package", "0.9.0")],
            new Dictionary<string, string> { ["Updated.Package"] = "1.0.0" });

        capture.Output.Should().Contain("Deleted references 1:");
        capture.Output.Should().Contain("Updated references 1:");
        capture.Output.Should().Contain("Added references 1:");
        capture.Output.Should().Contain("Not changed references 1:");
        capture.Output.Should().Contain("Removed.Package");
        capture.Output.Should().Contain("-> 2.0.0");
    }

    [Test]
    public void ShowGroupResult_prints_target_framework_header()
    {
        using var capture = ConsoleCapture.Attach();

        var result = new DependencyComparisonResult();
        result.NoChangesReferences.Add(new Dependency("MailKit", "4.16.0"));
        ConsoleHelper.ShowGroupResult("net8.0", result);

        capture.Output.Should().Contain("<group targetFramework=\"net8.0\">");
        capture.Output.Should().Contain("Not changed references");
    }

    [Test]
    public void Write_with_column_width_pads_output()
    {
        using var capture = ConsoleCapture.Attach();

        ConsoleHelper.Write("ab", ConsoleColor.Gray, columnWidth: 6);

        capture.Output.Should().Be("ab    ");
    }

    [TestCase("true", true)]
    [TestCase("1", true)]
    [TestCase("yes", true)]
    [TestCase("false", false)]
    [TestCase(null, false)]
    public void Write_with_ansi_env_emits_escape_sequences(string? envValue, bool ansiExpected)
    {
        Environment.SetEnvironmentVariable("CONSOLE_ANSI_COLOR", envValue);
        try
        {
            using var capture = ConsoleCapture.Attach();

            ConsoleHelper.WriteLine("line", ConsoleColor.Yellow);

            if (ansiExpected)
            {
                capture.Output.Should().Contain("\u001b[93m");
                capture.Output.Should().Contain("\u001b[0m");
            }
            else
            {
                capture.Output.Should().NotContain("\u001b[");
            }

            capture.Output.Should().Contain("line");
        }
        finally
        {
            Environment.SetEnvironmentVariable("CONSOLE_ANSI_COLOR", null);
        }
    }

    [Test]
    public void Write_with_ansi_emits_color_and_reset()
    {
        Environment.SetEnvironmentVariable("CONSOLE_ANSI_COLOR", "true");
        try
        {
            using var capture = ConsoleCapture.Attach();

            ConsoleHelper.Write("x", ConsoleColor.Cyan);

            capture.Output.Should().Contain("\u001b[96m");
            capture.Output.Should().Contain("x");
            capture.Output.Should().Contain("\u001b[0m");
        }
        finally
        {
            Environment.SetEnvironmentVariable("CONSOLE_ANSI_COLOR", null);
        }
    }

    [TestCase(ConsoleColor.Red, "\u001b[91m")]
    [TestCase(ConsoleColor.Green, "\u001b[92m")]
    [TestCase(ConsoleColor.Yellow, "\u001b[93m")]
    [TestCase(ConsoleColor.Cyan, "\u001b[96m")]
    [TestCase(ConsoleColor.Gray, "\u001b[90m")]
    [TestCase(ConsoleColor.Black, "\u001b[30m")]
    [TestCase(ConsoleColor.DarkBlue, "\u001b[34m")]
    [TestCase(ConsoleColor.DarkGreen, "\u001b[32m")]
    [TestCase(ConsoleColor.DarkCyan, "\u001b[36m")]
    [TestCase(ConsoleColor.DarkRed, "\u001b[31m")]
    [TestCase(ConsoleColor.DarkMagenta, "\u001b[35m")]
    [TestCase(ConsoleColor.DarkYellow, "\u001b[33m")]
    [TestCase(ConsoleColor.DarkGray, "\u001b[90m")]
    [TestCase(ConsoleColor.Blue, "\u001b[94m")]
    [TestCase(ConsoleColor.Magenta, "\u001b[95m")]
    [TestCase(ConsoleColor.White, "\u001b[97m")]
    public void Write_with_ansi_maps_console_colors(ConsoleColor color, string expectedSequence)
    {
        Environment.SetEnvironmentVariable("CONSOLE_ANSI_COLOR", "true");
        try
        {
            using var capture = ConsoleCapture.Attach();

            ConsoleHelper.Write("c", color);

            capture.Output.Should().Contain(expectedSequence);
        }
        finally
        {
            Environment.SetEnvironmentVariable("CONSOLE_ANSI_COLOR", null);
        }
    }

    [Test]
    public void Write_with_ansi_unknown_color_omits_color_sequence()
    {
        Environment.SetEnvironmentVariable("CONSOLE_ANSI_COLOR", "true");
        try
        {
            using var capture = ConsoleCapture.Attach();

            ConsoleHelper.Write("c", (ConsoleColor)999);

            capture.Output.Should().Be("c\u001b[0m");
        }
        finally
        {
            Environment.SetEnvironmentVariable("CONSOLE_ANSI_COLOR", null);
        }
    }

    [Test]
    public void WriteLine_in_dry_run_uses_neutral_color()
    {
        ConsoleHelper.DryRun = true;
        try
        {
            using var capture = ConsoleCapture.Attach();

            ConsoleHelper.WriteLine("deleted", ConsoleColor.Red);

            capture.Output.Should().NotContain("\u001b[91m");
            capture.Output.Should().Contain("deleted");
        }
        finally
        {
            ConsoleHelper.DryRun = false;
        }
    }

    [Test]
    public void WriteLine_in_dry_run_with_ansi_uses_gray_sequence()
    {
        Environment.SetEnvironmentVariable("CONSOLE_ANSI_COLOR", "true");
        ConsoleHelper.DryRun = true;
        try
        {
            using var capture = ConsoleCapture.Attach();

            ConsoleHelper.WriteLine("deleted", ConsoleColor.Red);

            capture.Output.Should().Contain("\u001b[90m");
            capture.Output.Should().NotContain("\u001b[91m");
        }
        finally
        {
            ConsoleHelper.DryRun = false;
            Environment.SetEnvironmentVariable("CONSOLE_ANSI_COLOR", null);
        }
    }

    [Test]
    public void DetermineColumnNameWidth_returns_longest_name_length()
    {
        var references = new List<Dependency>
        {
            new("Short", "1.0"),
            new("LongerName", "2.0"),
        };

        references.DetermineColumnNameWidth().Should().Be("LongerName".Length);
    }

    [Test]
    public void DetermineColumnVersionWidth_returns_longest_version_length()
    {
        var versions = new Dictionary<string, string>
        {
            ["a"] = "1.0.0",
            ["b"] = "10.0.0-preview.1",
        };

        versions.DetermineColumnVersionWidth().Should().Be("10.0.0-preview.1".Length);
    }
}
