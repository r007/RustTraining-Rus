# Containerized deployment

Optional, opt-in way to self-host the book collection without GitHub Pages —
useful behind a firewall or on an internal network.

**This is not the local development path.** For writing and previewing, use
`cargo xtask serve`, which rebuilds and serves at <http://localhost:3000> with
no container involved.

## Usage

From the repository root:

```bash
docker compose -f docker/compose.yaml up --build
```

Then open <http://localhost:3000>. Override the host port with `PORT`:

```bash
PORT=8080 docker compose -f docker/compose.yaml up --build
```

Without Compose:

```bash
docker build -f docker/Dockerfile -t rust-training .
docker run --rm -p 3000:8080 rust-training
```

Note the build context is the repository root in both cases — the build needs
the book sources and the `xtask` crate.

## How it works

Two stages:

1. **builder** (`rust:1-slim-bookworm`) installs `mdbook` and `mdbook-mermaid`,
   then runs `cargo xtask build`, which builds all seven books into `site/`
   along with the generated landing page.
2. **runtime** (`nginxinc/nginx-unprivileged:alpine`) serves `site/` on port
   8080. No Rust toolchain, no mdbook, no book sources in the final image.

`xtask build` is used rather than `xtask deploy` because the two produce
identical content — `deploy` only differs in writing to `docs/` and printing
GitHub Pages instructions, which are irrelevant in a container.

## Pinned versions

`MDBOOK_VERSION` and `MDBOOK_MERMAID_VERSION` are build args in the Dockerfile.
CI (`pages.yml`) currently installs both unpinned via `cargo install`, so the
container may lag or lead the published site after an upstream mdbook release.
Bump the args when that matters.

Prebuilt release binaries are used where upstream publishes them, falling back
to `cargo install` otherwise. As of the pinned versions, `mdbook-mermaid` has no
published arm64 Linux binary, so arm64 builds compile it from source and take
noticeably longer.

## Notes

- The container runs as uid 101 and binds an unprivileged port, so it needs no
  root and no added capabilities.
- Adding `read_only: true` to the service is possible but requires tmpfs mounts
  for nginx's cache and pid paths; it is left off by default rather than shipped
  untested.
- Content is baked in at build time. Rebuild the image to pick up book changes.
