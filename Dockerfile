# Runtime image; single-file publish from CI (artifacts/publish/linux-x64)
FROM mcr.microsoft.com/dotnet/runtime:8.0

LABEL maintainer="Denis Peshkov <denis.peshkov@outlook.com>"
LABEL repository="https://github.com/denis-peshkov/update-nuspec-action"
LABEL homepage="https://github.com/denis-peshkov/update-nuspec-action"

LABEL com.github.actions.name="Update *.nuspec"
LABEL com.github.actions.description="A Github action that scans .NET projects, and update in nuspec-file dependencies node."
LABEL com.github.actions.icon="activity"
LABEL com.github.actions.color="yellow"

COPY artifacts/publish/linux-x64/UpdateNuspecTool /UpdateNuspecTool
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /UpdateNuspecTool /entrypoint.sh

WORKDIR /github/workspace

ENTRYPOINT ["/entrypoint.sh"]
