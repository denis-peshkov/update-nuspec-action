# Image published to GHCR by CI. It packages the static musl binary built by the
# release-binaries matrix (no Rust build here). `action.yml` runs the pushed image
# via `image: docker://ghcr.io/...`, so consumers never build this Dockerfile.
#
# The binary must be staged at docker/update-nuspec before building:
#   cp <matrix>/update-nuspec docker/update-nuspec && docker build -t update-nuspec .
FROM alpine:3.20

LABEL maintainer="Denis Peshkov <denis.peshkov@outlook.com>"
LABEL repository="https://github.com/denis-peshkov/update-nuspec-action"
LABEL homepage="https://github.com/denis-peshkov/update-nuspec-action"

LABEL com.github.actions.name="Update *.nuspec"
LABEL com.github.actions.description="CLI to sync NuGet *.nuspec dependencies with PackageReference versions from matching *.csproj files."
LABEL com.github.actions.icon="activity"
LABEL com.github.actions.color="yellow"

COPY docker/update-nuspec /update-nuspec
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /update-nuspec /entrypoint.sh

WORKDIR /github/workspace

ENV CONSOLE_ANSI_COLOR=true

ENTRYPOINT ["/entrypoint.sh"]
