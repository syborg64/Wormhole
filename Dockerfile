FROM rust:1.78.0-buster AS builder
# Installation des dépendances système
RUN apt-get update && apt-get install -y \
    pkg-config \
    libfuse3-dev && \
    rm -rf /var/lib/apt/lists/*

# Création de l'utilisateur et configuration du répertoire
RUN useradd -m user && \
    mkdir -p /usr/src/wormhole && \
    chown -R user:user /usr/src/wormhole

USER user
WORKDIR /usr/src/wormhole
COPY --chown=user:user . .
RUN cargo build --release

FROM debian:bullseye-slim
# Installation minimale des dépendances
RUN apt-get update && apt-get install -y fuse3 && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -m user
USER user
WORKDIR /usr/src/wormhole/virtual
COPY --from=builder --chown=user:user /usr/src/wormhole/target/release/wormhole-service .
EXPOSE 8080
ENTRYPOINT [ "./wormhole-service" ]