namespace UpdateNuspecTool.Tests;

/// <summary>
/// <c>TestData/config.nuspec</c> (package id: Boilerplate.DataFilter).
/// </summary>
[TestFixture]
public sealed class BoilerplateDataFilterNuspecTests : NuspecFixtureTestBase
{
    protected override string NuspecFileName => "config.nuspec";

    [Test]
    public void Process_syncs_packages_from_Boilerplate_DataFilter_csproj()
    {
        var workspace = CreateWorkspace();
        RunProcess(workspace);
        var nuspecPath = GetNuspecPath(workspace);

        NuspecAssertionHelper.GetDependencyVersions(nuspecPath, "Cross.CQRS.EF")
            .Should()
            .ContainSingle()
            .Which
            .Should()
            .Be("7.0.0");

        NuspecAssertionHelper.GetDependencyVersions(nuspecPath, "Microsoft.EntityFrameworkCore")
            .Should()
            .ContainSingle()
            .Which
            .Should()
            .Be("6.123.47687");

        NuspecAssertionHelper.GetDependencyVersions(nuspecPath, "Boilerplate.WebApi.Contract")
            .Should()
            .ContainSingle()
            .Which
            .Should()
            .Be("13.5.77");

        NuspecAssertionHelper.ContainsDependency(nuspecPath, "Microsoft.AspNetCore.Authentication.JwtBearer")
            .Should()
            .BeTrue();

        NuspecAssertionHelper.ContainsDependency(nuspecPath, "AspNetCore.HealthChecks.Rabbitmq")
            .Should()
            .BeFalse();
    }
}
