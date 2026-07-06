# Publish tool inside the image (`uses: …@v1` does not require a pre-built binary in git)
FROM --platform=linux/amd64 rust:1-bookworm AS build

ARG VERSION=0.1.0
WORKDIR /src
COPY update-nuspec/ update-nuspec/
RUN sed -i "s/^version = .*/version = \"${VERSION}\"/" update-nuspec/Cargo.toml \
    && cargo build --release --manifest-path update-nuspec/Cargo.toml --bin update-nuspec

FROM --platform=linux/amd64 debian:bookworm-slim

LABEL maintainer="Denis Peshkov <denis.peshkov@outlook.com>"
LABEL repository="https://github.com/denis-peshkov/update-nuspec-action"
LABEL homepage="https://github.com/denis-peshkov/update-nuspec-action"

LABEL com.github.actions.name="Update *.nuspec"
LABEL com.github.actions.description="CLI to sync NuGet *.nuspec dependencies with PackageReference versions from matching *.csproj files."
LABEL com.github.actions.icon="activity"
LABEL com.github.actions.color="yellow"

COPY --from=build /src/update-nuspec/target/release/update-nuspec /update-nuspec
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /update-nuspec /entrypoint.sh

WORKDIR /github/workspace

ENV CONSOLE_ANSI_COLOR=true

ENTRYPOINT ["/entrypoint.sh"]
