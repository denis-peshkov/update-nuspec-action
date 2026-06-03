namespace UpdateNuspecTool.Tests;

[TestFixture]
public sealed class MyPackageNuspecTests : NuspecFixtureTestBase
{
    protected override string NuspecFileName => "MyPackage.nuspec";

    [Test]
    public void Process_syncs_NewtonsoftJson_from_csproj()
    {
        var workspace = CreateWorkspace();
        RunProcess(workspace);

        NuspecAssertionHelper
            .GetDependencyVersions(GetNuspecPath(workspace), "Newtonsoft.Json")
            .Should()
            .AllBeEquivalentTo("13.0.3");
    }
}
