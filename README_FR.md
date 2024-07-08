# WORMHOLE

## Définition des terms techniques

## Configuration

Le language choisi pour la configuration est le TOML, car il offre une très bonne clareté.

### [Configuration du réseau](./spec_fr/configuration/conf_principal.md)

Configuration générale du réseau
Clonée par les nouveaux arrivants lors d'une adhésion, elle définit les principales règles du réseau.
Elle est absolue, la configuration spécifique à un pod pouvant moduler mais non invalider son action.

### [Configuration par Pod](./spec_fr/configuration/pod_conf.md)

La configuration par pod n'est effective que pour ce pod.
Elle est néanmoins publique, pour aider le réseau à gérer l'ensemble des pods.
Ces règles ne sont appliquées que si leur existence n'invalide pas la configuration du réseau. ([voir](./spec/details/todo.md)) // TODO

## Architecture

### [Architecture Logique](./spec_fr/Architecture/architecture_logique.md)

// TODO

### [Architecture Code](./spec_fr/Architecture/architecture_code.md)

// TODO
