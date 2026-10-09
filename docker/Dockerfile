# syntax=docker/dockerfile:1
#
# Многоэтапная сборка для сборника книг RustTraining.
#
#   Этап 1 (builder) — тулчейн Rust, mdbook и mdbook-mermaid; запускает
#                      `cargo xtask build`, который создаёт site/.
#   Этап 2 (runtime) — непривилегированный nginx, раздающий готовые файлы.
#                      В итоговом образе нет Rust, mdbook и исходников.
#
# Собирайте из КОРНЯ РЕПОЗИТОРИЯ, а не из docker/:
#   docker build -f docker/Dockerfile -t rust-training .

ARG RUST_VERSION=1
ARG MDBOOK_VERSION=0.4.52
ARG MDBOOK_MERMAID_VERSION=0.14.0
ARG NGINX_VERSION=1.27

# ──────────────────────────────────────────────────────────────────────
# Этап 1: сборка книг
# ──────────────────────────────────────────────────────────────────────
FROM rust:${RUST_VERSION}-slim-bookworm AS builder

ARG MDBOOK_VERSION
ARG MDBOOK_MERMAID_VERSION
ARG TARGETARCH

WORKDIR /build

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# По возможности берём готовые бинарники: `cargo install mdbook mdbook-mermaid`
# компилирует оба инструмента из исходников и добавляет несколько минут
# к каждой холодной сборке.
#
# Upstream публикует не все архитектуры, поэтому при отсутствии бинарника
# выполняется компиляция:
#   - mdbook         есть для linux-gnu на amd64 и linux-musl на arm64
#   - mdbook-mermaid НЕ публикует бинарников для arm64 Linux вообще
# Статические сборки musl работают и на glibc-образе.
RUN set -eux; \
    case "${TARGETARCH:-amd64}" in \
        amd64) mdbook_triple=x86_64-unknown-linux-gnu;  mermaid_triple=x86_64-unknown-linux-gnu ;; \
        arm64) mdbook_triple=aarch64-unknown-linux-musl; mermaid_triple=aarch64-unknown-linux-musl ;; \
        *) echo "неподдерживаемый TARGETARCH: ${TARGETARCH}" >&2; exit 1 ;; \
    esac; \
    \
    install_tool() { \
        bin="$1"; url="$2"; crate="$3"; ver="$4"; \
        if curl -fsSL "$url" 2>/dev/null | tar -xz -C /usr/local/bin "$bin" 2>/dev/null; then \
            echo "==> $bin установлен из готового бинарника"; \
        else \
            echo "==> готового $bin для ${TARGETARCH} нет, компилируем из исходников"; \
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

# Копируем весь воркспейс. xtask определяет корень проекта через
# CARGO_MANIFEST_DIR во время компиляции, поэтому собирать и запускать его
# нужно на месте.
COPY . .

# `cargo xtask build` пишет в site/. Цель deploy (docs/) предназначена для
# GitHub Pages и дополнительно печатает инструкции по коммиту, здесь они не нужны.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo run --release --package xtask -- build \
    && test -f site/index.html \
    && echo "==> собрано книг: $(find site -mindepth 1 -maxdepth 1 -type d | wc -l)"

# ──────────────────────────────────────────────────────────────────────
# Этап 2: раздача
# ──────────────────────────────────────────────────────────────────────
# nginx-unprivileged — стандартный образ nginx, переконфигурированный на
# работу от uid 101 и порт 8080. Поэтому контейнеру не нужны root-права
# и NET_BIND_SERVICE.
FROM nginxinc/nginx-unprivileged:${NGINX_VERSION}-alpine AS runtime

COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder --chown=nginx:nginx /build/site /usr/share/nginx/html

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD wget -q -O /dev/null http://127.0.0.1:8080/ || exit 1
