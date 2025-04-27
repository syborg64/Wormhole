FROM rust:1.78.0-buster AS builder

# Installer les dépendances système
RUN apt update && apt install -y \
    pkg-config \
    libfuse3-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/wormhole
COPY . .

# Construire les deux binaires
RUN cargo build --bin wormhole-service && \
    cargo build --bin wormhole-cli

FROM debian:bullseye-slim

# Dépendances minimales
RUN apt-get update --no-install-recommends && \
    apt-get install -y --no-install-suggests fuse3 netcat-openbsd systemd && \
    rm -rf /var/lib/apt/lists/*

# Configuration FUSE
RUN echo 'user_allow_other' | tee -a /etc/fuse.conf

WORKDIR /usr/src/wormhole

# Créer le dossier utilisé par l'app
RUN mkdir -p /usr/src/wormhole/virtual && \
    chmod -R 775 /usr/src/wormhole/virtual

# Copier les deux binaires depuis le builder
COPY --from=builder /usr/src/wormhole/target/debug/wormhole-service .
COPY --from=builder /usr/src/wormhole/target/debug/wormhole-cli .

COPY wormhole-service.service /etc/systemd/system/wormhole-service.service

RUN systemctl enable wormhole-service.service

CMD ["/bin/systemd"]
