FROM rust:1.83-bookworm

# gfortran 12 (bookworm) and python3 are the pinned baseline toolchain.
RUN apt-get update \
    && apt-get install -y --no-install-recommends gfortran python3 patch \
    && rm -rf /var/lib/apt/lists/*

ENV CARGO_TERM_COLOR=always
WORKDIR /work
