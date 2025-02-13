#!/bin/bash

# Créer les dossiers s'ils n'existent pas
mkdir -p shared_mnt1 shared_mnt2 shared_mnt3

# Vérifier si l'image Docker a déjà été construite
if docker images | grep -q "wormhole-base"; then
    echo "L'image Docker existe déjà. Lancement de docker-compose up --no-deps..."
    kitty --hold bash -c "docker-compose up --no-deps"
else
    echo "L'image Docker n'a jamais été construite. Lancement de docker-compose up --build --no-deps..."
    kitty --hold bash -c "docker-compose up --build --no-deps"
fi
