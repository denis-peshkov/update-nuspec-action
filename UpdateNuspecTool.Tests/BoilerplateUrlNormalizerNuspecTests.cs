namespace UpdateNuspecTool.Tests;

/// <summary>
/// <c>TestData/cgf.nuspec</c> (package id: Boilerplate.Url.Normalizer).
/// </summary>
[TestFixture]
public sealed class BoilerplateUrlNormalizerNuspecTests : NuspecFixtureTestBase
{
    protected override string NuspecFileName => "cgf.nuspec";

    [Test]
    public void Process_syncs_packages_from_Boilerplate_Url_Normalizer_csproj()
    {
        var workspace = CreateWorkspace();
        RunProcess(workspace);
        var nuspecPath = GetNuspecPath(workspace);

        NuspecAssertionHelper.GetDependencyVersions(nuspecPath, "Cross.CQRS.EF")
            .Should()
            .ContainSingle()
            .Which
            .Should()
            .Be("8.8885.22-preview");

        NuspecAssertionHelper.GetDependencyVersions(nuspecPath, "Boilerplate.WebApi.Contract")
            .Should()
            .ContainSingle()
            .Which
            .Should()
            .Be("22.22.00");

        NuspecAssertionHelper.GetDependencyVersions(nuspecPath, "Microsoft.EntityFrameworkCore")
            .Should()
            .ContainSingle()
            .Which
            .Should()
            .Be("5.234");

        NuspecAssertionHelper.GetDependencyVersions(nuspecPath, "Microsoft.AspNetCore.Http.Abstractions")
            .Should()
            .ContainSingle()
            .Which
            .Should()
            .Be("3.0.0.12");

        NuspecAssertionHelper.GetDependencyVersions(nuspecPath, "MailKit")
            .Should()
            .ContainSingle()
            .Which
            .Should()
            .Be("3.2.0");

        NuspecAssertionHelper.ContainsDependency(nuspecPath, "Serilog.AspNetCore")
            .Should()
            .BeFalse();
    }
}
