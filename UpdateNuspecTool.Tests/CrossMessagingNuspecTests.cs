namespace UpdateNuspecTool.Tests;

[TestFixture]
public sealed class CrossMessagingNuspecTests : NuspecFixtureTestBase
{
    protected override string NuspecFileName => "Cross.Messaging.nuspec";

    [Test]
    public void Process_syncs_packages_from_csproj_in_all_groups()
    {
        var workspace = CreateWorkspace();
        RunProcess(workspace);
        var nuspecPath = GetNuspecPath(workspace);

        NuspecAssertionHelper
            .GetDependencyVersions(nuspecPath, "MailKit")
            .Should()
            .NotBeEmpty()
            .And
            .AllBeEquivalentTo("4.17.0");

        NuspecAssertionHelper
            .GetDependencyVersions(nuspecPath, "Microsoft.Extensions.Configuration")
            .Should()
            .AllBeEquivalentTo("8.0.1");

        NuspecAssertionHelper.ContainsDependency(nuspecPath, "Microsoft.Extensions.Options.TTTT")
            .Should()
            .BeFalse();
    }
}
