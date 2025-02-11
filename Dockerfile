FROM rust:1.78.0-buster AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libfuse3-dev  # Pour FUSE3
RUN rm -rf /var/lib/apt/lists/*

RUN useradd -m user
USER user

# Créez un répertoire pour l'application
WORKDIR /usr/src/wormhole

# Copiez les fichiers de votre projet dans le conteneur
COPY --chown=user:user . .

# Construisez l'application
RUN cargo build --release

# Utilisez une image légère pour l'exécution
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    fuse3
RUN rm -rf /var/lib/apt/lists/*

RUN useradd -m user
USER user

WORKDIR /usr/src/wormhole/virtual

# Copiez le binaire construit depuis l'étape de construction
COPY --from=builder --chown=user:user /usr/src/wormhole/target/release/wormhole-service /usr/src/wormhole/wormhole-service

# Exposez le port sur lequel votre service écoute
EXPOSE 8080

# Commande par défaut pour lancer l'application
ENTRYPOINT [ "./wormhole-service" ]
