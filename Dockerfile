# Build the runtime image
FROM mcr.microsoft.com/dotnet/runtime:8.0

# Label the container
LABEL maintainer="Denis Peshkov <denis.peshkov@outlook.com>"
LABEL repository="https://github.com/denis-peshkov/update-nuspec-action"
LABEL homepage="https://github.com/denis-peshkov/update-nuspec-action"

LABEL com.github.actions.name="Update *.nuspec"
LABEL com.github.actions.description="A Github action that scans .NET projects, and update nuspec-file."
LABEL com.github.actions.icon="activity"
LABEL com.github.actions.color="yellow"

COPY /tools/UpdateNuspecTool .

ENTRYPOINT [ "dotnet", "/UpdateNuspecTool" ]
