namespace UpdateNuspecTool;

public static class ConsoleHelper
{
    private static ConsoleColor _deletedColor = ConsoleColor.Red;
    private static ConsoleColor _updatedColor = ConsoleColor.Green;
    private static ConsoleColor _addedColor = ConsoleColor.Yellow;
    private static ConsoleColor _notChangedColor = ConsoleColor.Gray;

    public static void Write(string text, ConsoleColor color, int columnWidth = 0)
    {
        Console.ForegroundColor = color;
        Console.Write(text.PadRight(columnWidth));
        Console.ResetColor();
    }

    public static void WriteLine(string text, ConsoleColor color)
    {
        Console.ForegroundColor = color;
        Console.WriteLine(text);
        Console.ResetColor();
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
        var columnWidth = columnWidthHelperList.DetermineColumnNameWidth() + 5;

        if (deletedReferences.Any())
        {
            WriteLine($"Deleted references {deletedReferences.Count}:", _deletedColor);
            foreach (var item in deletedReferences)
            {
                Write($"\t Name:", ConsoleColor.Gray);
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
            WriteLine($"Updated references {updatedReferences.Count}:", _updatedColor);
            foreach (var item in updatedReferences)
            {
                Write($"\t Name:", ConsoleColor.Gray);
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
            WriteLine($"Added references {addedReferences.Count}:", _addedColor);
            foreach (var item in addedReferences)
            {
                Write($"\t Name:", ConsoleColor.Gray);
                Write($" {item.Name}", _addedColor, columnWidth);
                Write($"Version: ", ConsoleColor.Gray);
                Write($"{item.Version}", _addedColor);
                Console.WriteLine();
            }

            Console.WriteLine();
        }

        if (noChangesReferences.Any())
        {
            Console.WriteLine($"Not changed references {noChangesReferences.Count}:");
            foreach (var item in noChangesReferences)
            {
                Write($"\t Name:", ConsoleColor.Gray);
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
        return references.Select(p => p.Name).Aggregate("", (max, cur) => max.Length > cur.Length ? max : cur)
            .Length;
    }

    //TODO: refactor
    public static int DetermineColumnVersionWidth(this IDictionary<string, string> references)
    {
        return references.Select(p => p.Value).Aggregate("", (max, cur) => max.Length > cur.Length ? max : cur)
            .Length;
    }
}
