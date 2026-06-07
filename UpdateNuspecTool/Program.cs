var options = CliHelper.ParseArgs(args);

if (options.ShowHelp)
{
    CliHelper.PrintHelp();
    return;
}

if (options.ShowVersion)
{
    CliHelper.PrintVersion();
    return;
}

CliHelper.Run(options);
