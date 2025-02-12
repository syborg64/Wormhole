FROM rust:1.78.0-buster AS builder
# Installation des dépendances système
RUN apt update && apt install -y \
    pkg-config \
    libfuse3-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/wormhole
COPY --chown=root:root . .
RUN cargo build

# Création de l'utilisateur et configuration du répertoire
RUN useradd -m user && \
    groupadd -r fuse && \
    usermod -a -G fuse user && \
    mkdir -p /usr/src/wormhole/virtual && \
    chown -R user:fuse /usr/src/wormhole/virtual && \
    chmod -R 775 /usr/src/wormhole/virtual

USER user

FROM debian:bullseye-slim
# Installation minimale des dépendances
RUN apt-get update --no-install-recommends && \
    apt-get install -y --no-install-suggests fuse3 && \
    rm -rf /var/lib/apt/lists/*

# Configuration FUSE
RUN echo 'user_allow_other' | tee -a /etc/fuse.conf

WORKDIR /usr/src/wormhole
RUN useradd -m user && \
    groupadd -r fuse && \
    usermod -a -G fuse user && \
    mkdir -p /usr/src/wormhole/virtual && \
    chown -R user:fuse /usr/src/wormhole/virtual && \
    chmod -R 775 /usr/src/wormhole/virtual

USER user
COPY --from=builder --chown=user:user /usr/src/wormhole/target/debug/wormhole-service .
RUN ls -la /usr/src/wormhole