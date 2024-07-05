# WORMHOLE

## Définition des termes techniques


## Configuration
Le langage choisi pour la configuration est le langage TOML, offrant une très bonne clareté.

### [Configuration du réseau](./configuration/main_conf.md)
Configuration générale du réseau
Clonée par les arrivants lors d'un join, elle définit les règles principales du réseau.
Elle est absolue, la configuration pod-specific pouvant moduler mais non invalider son action.

### [Configuration par Pod](./configuration/pod_conf.md)
La configuration par pod est une configuration effective uniquement pour celui-ci.
Elle est tout de même publique pour aider le réseau à gérer l'ensemble des pods.
Ces règles sont uniquement appliquées si leur existance n'invalide pas la configuration du réseau ([voir](./details/todo.md)) // TODO

## Architecture

### Archtecture logique
// TODO

### Architecture code
// TODO
