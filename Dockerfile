FROM rust:latest as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libc-bin    \
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
FROM debian:buster-slim

RUN apt-get update && apt-get install -y \
    libc-bin    \
    fuse3
RUN rm -rf /var/lib/apt/lists/*

RUN useradd -m user
USER user


# Copiez le binaire construit depuis l'étape de construction
COPY --from=builder --chown=user:user /usr/src/wormhole/target/release/wormhole-service /usr/local/bin/wormhole-service

# Exposez le port sur lequel votre service écoute
EXPOSE 8080

# Commande par défaut pour lancer l'application
CMD ["wormhole-service", "${WH_ADDRESS}", "${WH_PEERS}", "${WH_MOUNT_POINT}"]
