namespace UpdateNuspecTool.Helper;

public static class ConsoleHelper
{
    private const string AnsiReset = "\u001b[0m";

    private static ConsoleColor _deletedColor = ConsoleColor.Red;
    private static ConsoleColor _updatedColor = ConsoleColor.Yellow;
    private static ConsoleColor _addedColor = ConsoleColor.Green;
    private static ConsoleColor _notChangedColor = ConsoleColor.Gray;

    public static bool DryRun { get; set; }

    public static void Write(string text, ConsoleColor color, int columnWidth = 0)
    {
        color = EffectiveColor(color);
        var output = columnWidth > 0 ? text.PadRight(columnWidth) : text;
        if (IsAnsiEnabled())
        {
            Console.Write($"{ToAnsi(color)}{output}{AnsiReset}");
            return;
        }

        Console.ForegroundColor = color;
        Console.Write(output);
        Console.ResetColor();
    }

    public static void WriteLine(string text, ConsoleColor color)
    {
        color = EffectiveColor(color);
        if (IsAnsiEnabled())
        {
            Console.WriteLine($"{ToAnsi(color)}{text}{AnsiReset}");
            return;
        }

        Console.ForegroundColor = color;
        Console.WriteLine(text);
        Console.ResetColor();
    }

    public static void ShowGroupResult(string targetFramework, DependencyComparisonResult result)
    {
        WriteLine($"<group targetFramework=\"{targetFramework}\">", ConsoleColor.Cyan);
        ShowResult(result);
    }

    public static void ShowResult(DependencyComparisonResult result)
    {
        ShowResult(
            result.UpdatedReferences,
            result.NoChangesReferences,
            result.AddedReferences,
            result.DeletedReferences,
            result.OutdatedReferences);
    }

    public static void ShowResult(
        List<Dependency> updatedReferences,
        List<Dependency> noChangesReferences,
        List<Dependency> addedReferences,
        List<Dependency> deletedReferences,
        IDictionary<string, string> outdatedReferences)
    {
        var columnWidthHelperList = new List<Dependency>();
        columnWidthHelperList.AddRange(updatedReferences);
        columnWidthHelperList.AddRange(noChangesReferences);
        columnWidthHelperList.AddRange(addedReferences);
        columnWidthHelperList.AddRange(deletedReferences);

        if (!columnWidthHelperList.Any())
        {
            WriteLine("\t (no dependency changes)", ConsoleColor.Gray);
            Console.WriteLine();
            return;
        }

        var columnWidth = columnWidthHelperList.DetermineColumnNameWidth() + 5;

        if (deletedReferences.Any())
        {
            WriteLine($"\t Deleted references {deletedReferences.Count}:", _deletedColor);
            foreach (var item in deletedReferences)
            {
                Write($"\t\t Name:", ConsoleColor.Gray);
                Write($" {item.Name}", _deletedColor, columnWidth);
                Write($"Version: ", ConsoleColor.Gray);
                Write($"{item.Version}", _deletedColor);
                Console.WriteLine();
            }

            Console.WriteLine();
        }

        if (updatedReferences.Any())
        {
            var columnVersionWidth = outdatedReferences.DetermineColumnVersionWidth() + 5;
            WriteLine($"\t Updated references {updatedReferences.Count}:", _updatedColor);
            foreach (var item in updatedReferences)
            {
                Write($"\t\t Name:", ConsoleColor.Gray);
                Write($" {item.Name}", _updatedColor, columnWidth);
                Write($"Version: ", ConsoleColor.Gray);
                Write($"{outdatedReferences[item.Name]} ", ConsoleColor.Gray, columnVersionWidth);
                Write($"-> {item.Version}", _updatedColor);
                Console.WriteLine();
            }

            Console.WriteLine();
        }

        if (addedReferences.Any())
        {
            WriteLine($"\t Added references {addedReferences.Count}:", _addedColor);
            foreach (var item in addedReferences)
            {
                Write($"\t\t Name:", ConsoleColor.Gray);
                Write($" {item.Name}", _addedColor, columnWidth);
                Write($"Version: ", ConsoleColor.Gray);
                Write($"{item.Version}", _addedColor);
                Console.WriteLine();
            }

            Console.WriteLine();
        }

        if (noChangesReferences.Any())
        {
            WriteLine($"\t Not changed references {noChangesReferences.Count}:", ConsoleColor.Gray);
            foreach (var item in noChangesReferences)
            {
                Write($"\t\t Name:", ConsoleColor.Gray);
                Write($" {item.Name}", _notChangedColor, columnWidth);
                Write($"Version: ", ConsoleColor.Gray);
                Write($"{item.Version}", _notChangedColor);
                Console.WriteLine();
            }

            Console.WriteLine();
        }
    }

    public static int DetermineColumnNameWidth(this List<Dependency> references)
    {
        return references
            .Select(p => p.Name)
            .Aggregate("", (max, cur) => max.Length > cur.Length ? max : cur)
            .Length;
    }

    public static int DetermineColumnVersionWidth(this IDictionary<string, string> references)
    {
        return references
            .Select(p => p.Value)
            .Aggregate("", (max, cur) => max.Length > cur.Length ? max : cur)
            .Length;
    }

    private static ConsoleColor EffectiveColor(ConsoleColor color)
        => DryRun
            ? _notChangedColor
            : color;

    private static bool IsAnsiEnabled()
    {
        var value = Environment.GetEnvironmentVariable("CONSOLE_ANSI_COLOR");
        return value is not null
            && (value.Equals("true", StringComparison.OrdinalIgnoreCase)
                || value == "1"
                || value.Equals("yes", StringComparison.OrdinalIgnoreCase));
    }

    private static string ToAnsi(ConsoleColor color) => color switch
    {
        ConsoleColor.Black => "\u001b[30m",
        ConsoleColor.DarkBlue => "\u001b[34m",
        ConsoleColor.DarkGreen => "\u001b[32m",
        ConsoleColor.DarkCyan => "\u001b[36m",
        ConsoleColor.DarkRed => "\u001b[31m",
        ConsoleColor.DarkMagenta => "\u001b[35m",
        ConsoleColor.DarkYellow => "\u001b[33m",
        ConsoleColor.Gray => "\u001b[90m",
        ConsoleColor.DarkGray => "\u001b[90m",
        ConsoleColor.Blue => "\u001b[94m",
        ConsoleColor.Green => "\u001b[92m",
        ConsoleColor.Cyan => "\u001b[96m",
        ConsoleColor.Red => "\u001b[91m",
        ConsoleColor.Magenta => "\u001b[95m",
        ConsoleColor.Yellow => "\u001b[93m",
        ConsoleColor.White => "\u001b[97m",
        _ => string.Empty,
    };
}
