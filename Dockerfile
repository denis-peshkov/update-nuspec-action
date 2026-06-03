# Build UpdateNuspecTool for linux-x64 (GitHub Actions runner)
FROM mcr.microsoft.com/dotnet/sdk:8.0 AS build
WORKDIR /src

ARG BUILD_CONFIG=Release
ARG VERSION=0.2.0
ARG ASSEMBLY_VERSION=0.2.0.0
ARG FILE_VERSION=0.2.0.0
ARG INFORMATIONAL_VERSION=0.2.0
ARG COMPANY=Denis Peshkov
ARG PRODUCT=update-nuspec-action
ARG DESCRIPTION=GitHub Action that syncs NuGet dependencies in *.nuspec with PackageReference from matching *.csproj files.
ARG REPOSITORY_URL=https://github.com/denis-peshkov/update-nuspec-action.git
ARG REPOSITORY_TYPE=git
ARG CLS_COMPLIANT=true
ARG NEUTRAL_LANGUAGE=en

COPY UpdateNuspecTool/ UpdateNuspecTool/

RUN dotnet publish UpdateNuspecTool/UpdateNuspecTool.csproj \
    --configuration "${BUILD_CONFIG}" \
    -r linux-x64 \
    --self-contained false \
    -p:Configuration="${BUILD_CONFIG}" \
    -p:Version="${VERSION}" \
    -p:AssemblyVersion="${ASSEMBLY_VERSION}" \
    -p:FileVersion="${FILE_VERSION}" \
    -p:InformationalVersion="${INFORMATIONAL_VERSION}" \
    -p:Company="${COMPANY}" \
    -p:Product="${PRODUCT}" \
    -p:Description="${DESCRIPTION}" \
    -p:RepositoryUrl="${REPOSITORY_URL}" \
    -p:RepositoryType="${REPOSITORY_TYPE}" \
    -p:CLSCompliant="${CLS_COMPLIANT}" \
    -p:NeutralLanguage="${NEUTRAL_LANGUAGE}" \
    -p:PublishSingleFile=true \
    -o /app/publish

# Runtime image for the action
FROM mcr.microsoft.com/dotnet/runtime:8.0

LABEL maintainer="Denis Peshkov <denis.peshkov@outlook.com>"
LABEL repository="https://github.com/denis-peshkov/update-nuspec-action"
LABEL homepage="https://github.com/denis-peshkov/update-nuspec-action"

LABEL com.github.actions.name="Update *.nuspec"
LABEL com.github.actions.description="A Github action that scans .NET projects, and update in nuspec-file dependencies node."
LABEL com.github.actions.icon="activity"
LABEL com.github.actions.color="yellow"

COPY --from=build /app/publish/UpdateNuspecTool /UpdateNuspecTool
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /UpdateNuspecTool /entrypoint.sh

WORKDIR /github/workspace

ENTRYPOINT ["/entrypoint.sh"]
