# syntax=docker/dockerfile:1
#
# Multi-stage build for the RustTraining book collection.
#
#   Stage 1 (builder) — Rust toolchain + mdbook + mdbook-mermaid, runs
#                       `cargo xtask build` to produce site/.
#   Stage 2 (runtime) — unprivileged nginx serving the static output.
#                       No Rust, no mdbook, no source in the final image.
#
# Build from the REPOSITORY ROOT, not from docker/:
#   docker build -f docker/Dockerfile -t rust-training .

ARG RUST_VERSION=1
ARG MDBOOK_VERSION=0.4.52
ARG MDBOOK_MERMAID_VERSION=0.14.0
ARG NGINX_VERSION=1.27

# ──────────────────────────────────────────────────────────────────────
# Stage 1: build the books
# ──────────────────────────────────────────────────────────────────────
FROM rust:${RUST_VERSION}-slim-bookworm AS builder

ARG MDBOOK_VERSION
ARG MDBOOK_MERMAID_VERSION
ARG TARGETARCH

WORKDIR /build

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Prefer prebuilt release binaries — `cargo install mdbook mdbook-mermaid`
# compiles both from source and adds several minutes to every cold build.
#
# Upstream does not ship a complete set of prebuilt targets, so we fall back
# to compiling when an asset is missing:
#   - mdbook         has linux-gnu on amd64, linux-musl on arm64
#   - mdbook-mermaid has NO published arm64 Linux binary at all
# musl builds are statically linked and run fine on this glibc base image.
RUN set -eux; \
    case "${TARGETARCH:-amd64}" in \
        amd64) mdbook_triple=x86_64-unknown-linux-gnu;  mermaid_triple=x86_64-unknown-linux-gnu ;; \
        arm64) mdbook_triple=aarch64-unknown-linux-musl; mermaid_triple=aarch64-unknown-linux-musl ;; \
        *) echo "unsupported TARGETARCH: ${TARGETARCH}" >&2; exit 1 ;; \
    esac; \
    \
    install_tool() { \
        bin="$1"; url="$2"; crate="$3"; ver="$4"; \
        if curl -fsSL "$url" 2>/dev/null | tar -xz -C /usr/local/bin "$bin" 2>/dev/null; then \
            echo "==> installed $bin from prebuilt binary"; \
        else \
            echo "==> no prebuilt $bin for ${TARGETARCH}, compiling from source"; \
            cargo install "$crate" --version "$ver" --locked --root /usr/local; \
        fi; \
    }; \
    \
    install_tool mdbook \
        "https://github.com/rust-lang/mdBook/releases/download/v${MDBOOK_VERSION}/mdbook-v${MDBOOK_VERSION}-${mdbook_triple}.tar.gz" \
        mdbook "${MDBOOK_VERSION}"; \
    install_tool mdbook-mermaid \
        "https://github.com/badboy/mdbook-mermaid/releases/download/v${MDBOOK_MERMAID_VERSION}/mdbook-mermaid-v${MDBOOK_MERMAID_VERSION}-${mermaid_triple}.tar.gz" \
        mdbook-mermaid "${MDBOOK_MERMAID_VERSION}"; \
    \
    mdbook --version; \
    mdbook-mermaid --version

# Copy the whole workspace. xtask resolves the project root from
# CARGO_MANIFEST_DIR at compile time, so it must be built and run in place.
COPY . .

# `cargo xtask build` writes to site/ (the deploy target, docs/, is for
# GitHub Pages and additionally prints commit instructions we don't want here).
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo run --release --package xtask -- build \
    && test -f site/index.html \
    && echo "==> built $(find site -mindepth 1 -maxdepth 1 -type d | wc -l) books"

# ──────────────────────────────────────────────────────────────────────
# Stage 2: serve
# ──────────────────────────────────────────────────────────────────────
# nginx-unprivileged is the stock nginx image reconfigured to run as uid 101
# and listen on 8080, so the container needs no root and no NET_BIND_SERVICE.
FROM nginxinc/nginx-unprivileged:${NGINX_VERSION}-alpine AS runtime

COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder --chown=nginx:nginx /build/site /usr/share/nginx/html

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD wget -q -O /dev/null http://127.0.0.1:8080/ || exit 1
