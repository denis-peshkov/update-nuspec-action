namespace UpdateNuspecTool.Tests;

[TestFixture]
public sealed class CrossMessagingNuspecTests : NuspecFixtureTestBase
{
    protected override string NuspecFileName => "Cross.Messaging.nuspec";

    [Test]
    public void Process_syncs_packages_per_target_framework_from_csproj()
    {
        var workspace = CreateWorkspace();
        RunProcess(workspace);
        var nuspecPath = GetNuspecPath(workspace);

        NuspecAssertionHelper
            .GetDependencyVersions(nuspecPath, "MailKit")
            .Should()
            .AllBeEquivalentTo("4.16.0");

        NuspecAssertionHelper.GetDependencyVersionInGroup(nuspecPath, "net8.0", "Microsoft.Extensions.Configuration")
            .Should()
            .Be("8.0.0");

        NuspecAssertionHelper.GetDependencyVersionInGroup(nuspecPath, "net9.0", "Microsoft.Extensions.Configuration")
            .Should()
            .Be("9.0.15");

        NuspecAssertionHelper.GetDependencyVersionInGroup(nuspecPath, "net10.0", "Microsoft.Extensions.Configuration")
            .Should()
            .Be("10.0.7");

        NuspecAssertionHelper.ContainsDependency(nuspecPath, "Microsoft.Extensions.Options.TTTT")
            .Should()
            .BeFalse();

        NuspecAssertionHelper.ContainsDependency(nuspecPath, "Microsoft.Extensions.Options.RRRRRR")
            .Should()
            .BeFalse();

        NuspecAssertionHelper.GetDependencyVersionInGroup(nuspecPath, "net7.0", "MailKit")
            .Should()
            .Be("4.16.0");

        NuspecAssertionHelper.GetDependencyVersionInGroup(nuspecPath, "net6.0", "Microsoft.Extensions.Configuration.Binder")
            .Should()
            .Be("8.0.2");

        NuspecAssertionHelper.GetDependencyVersionInGroup(nuspecPath, "net9.0", "Microsoft.Extensions.Configuration.Binder")
            .Should()
            .BeNull();
    }
}
