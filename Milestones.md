# Milestones

## Produit minimum viable (MVP)
Formation d'un réseau fonctionnel basique (plus de deux nodes) intégré nativement sur Linux (fonctions basiques comme créer, lire, modifier ou supprimer)
Possibilité d'ajouter de nouveaux pods ou de les supprimer sans interrompre le réseau (Hors cas particulier)

<!--
>    - connection automatique aux autres nodes
>    - ajout & suppression des pods
>    - création, modification, lecture et suppression de fichier sur le réseau
>    - déplacer des fichiers entre les pods
>    - intégration native linux
-->

## Consolidation
Le fonctionnement sera amélioré pour fluidifier l'experience et anticiper les fonctions futures.
Le cluster harmonisera son fonctionnement interne, (pourra relocaliser les données de Pod à Pod, leur localisation réelle sera traquée pour cibler les demandes au bon endroit, etc...)

<!--
Stocker de manière locale des informations sur les fichiers provenant des autres nodes.
Gérer les fichiers à l'échelle du cluster (relocalisation pod à pod si besion)


>   - information locale persistante des fichiers distants
>   - fonctionnalités d'aisance (importation)
-->

## Configuration

Créez un système de configuration basique pour le réseau et chaque pod, qui sera affiné avec l'ajout de nouvelles fonctionnalités.

<!-- Créer le systèmes de fichiers de configuration basic pour le réseau et pour chaque pod de l'instance. Ces configuration seront ensuite complétées au fure et à mesure du rajout de nouvelles fonctionnalités.

>    - configuration par réseau
>    - configuration par pod
>    - template de configuration -->

## Replication et cache

Permettez aux utilisateurs de répliquer des fichiers avec différents RAID et de déplacer les fichiers répliqués vers d'autres pods. Les fichiers répliqués seront stockés sur les pods les plus appropriés en fonction de la configuration de redondance. Les utilisateurs pourront également stocker des fichiers dans le cache de leur nœud pour un accès plus rapide.

<!-- Donner la possibilité aux utilisateurs de faire de la replications de fichiers avec différent RAID par exemple. En plus de la replication avoir la possibilité de déplacer les fichiers répliqué dans un autre pod que celui d'origine. Les fichiers répliqué pourrait être par default (si la configuration demande de la redondance) sur les pod dont la node est plus approprier (par exemple un ordinateur avec une disque HDD).
Si l'utilisateur souhaite accéder très rapidement à certains fichiers, sans les stocker sur son disque, il aurait la possibilité de les stocker dans le cache de sa node.

>    - cache en stockage local
>    - possession
>    - replication N
>    - déplacement de fichiers répliqués
>    - Affinité par pod pour la redondance -->

## Gestion des pannes

Mettez en place un système de base pour la gestion des pannes de cluster, qui servira de base à d'autres modules de gestion.

<!-- Mettre en place un système de base pour les gestions de pannes de cluster sur lequel d'autre module de gestion pourront être créé par la suite.

>    - identifier l'état de santé du réseau
>    - notification et log -->

## Intégration d'API

Assurez la compatibilité de Wormhole avec d'autres systèmes d'exploitation tels que Windows et Docker.

<!-- Faire en sorte de Wormhole fonction ailleurs que sur Linux, comme sur Windows ou Docker.

>    - fonctionalités équivalentes avec la version FUSE
>    - integration windows
>    - Intégration Docker / Kubernetes -->

## Stratégies de récupérations

En cas d'arrêt involontaire d'un pod, Wormhole rééquilibre la redondance des données, resynchronise le pod déconnecté et résout les conflits entre différentes versions d'un même fichier.

<!-- En cas d'arrêt involontaire d'un pod, Wormhole pourra rééquilibrer la redondance des données, resyncroniser le pod qui à été déconnecter et résoudre les conflits qu'il pourrait y avoir entre plusieurs version d'un même fichier.

>    - compensation / rééquilibrage
>    - conciliation de conflicts
>    - systèmes de récupération
>    - resynchronisation -->

## Amélioration des performance (leste)

Améilorer les perfomrances de Wormhole pour être plus rapide et moins énergivore en resource.

<!-- >    - tests de performances
>    - refactorations majeures 
>    - identification de code hot & lent -->

## Équilibrage du système

Mettez en place un système d'équilibrage pour nettoyer automatiquement le cache, suivre l'utilisation des fichiers et déplacer automatiquement les fichiers vers le pod le plus demandé. Cela permettra d'optimiser l'utilisation des ressources et de garantir une distribution équilibrée des données sur le réseau.

<!-- >    - nettoyage du cache automatisé
>    - track les accès et utilisations de fichier
>    - déplacement automatique des fichiers vers le pod le plus demandant -->

## Sécurité

Mettez en place une gestion d'erreur complète et verbeuse pour renforcer la sécurité. Cela permettra de détecter rapidement les erreurs et de prendre des mesures correctives pour éviter les violations de données et les temps d'arrêt.

<!-- >    - gestion d'erreur intégrale et verbeuse -->

## Authentication

Permettez aux utilisateurs de se connecter au réseau Wormhole avec différents niveaux de privilèges en utilisant l'authentification TLS/paire de clés ou SSH.

<!-- Permettre aux utilisateur de se connecter au réseau Wormhole permettant ainsi d'avoir des prvilèges différent en fonction des utilisateurs.

>    - authentification TLS/paire de clé
>    - par ssh -->
