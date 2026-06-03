# Build UpdateNuspecTool for linux-x64 (GitHub Actions runner)
FROM mcr.microsoft.com/dotnet/sdk:8.0 AS build
WORKDIR /src
COPY UpdateNuspecTool/ UpdateNuspecTool/
RUN dotnet publish UpdateNuspecTool/UpdateNuspecTool.csproj \
    -c Release \
    -r linux-x64 \
    --self-contained false \
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
