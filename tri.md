# Innovation Track

## Context

Le projet Wormhole est né de la nécessité de simplifier l'accès et la gestion des données entre plusieurs datacenters. Actuellement, les entreprises sont confrontées à plusieurs défis liés à la centralisation ou à la décentralisation des données. Les solutions centralisées présentent des risques de sécurité, des limitations d'accès et un stress accru sur l'infrastructure. En revanche, les infrastructures décentralisées sont difficiles à mettre en place et manquent de solutions universelles.

Wormhole se positionne comme une solution innovante visant à offrir un accès sécurisé, souverain, et facile aux données. Le projet vise à répondre aux besoins de sécurité, de flexibilité et de simplicité de configuration pour les entreprises et les particuliers. L'objectif est de fournir une infrastructure de données distribuée qui peut s'adapter à divers besoins, tout en garantissant la transparence et la sécurité des données.

Avec Wormhole, les utilisateurs finaux bénéficieront d'une expérience fluide et transparente. Les fichiers apparaîtront et se comporteront comme des fichiers locaux habituels, sans nécessiter de gestion complexe. Une fois le système mis en place, tout fonctionne automatiquement, garantissant une utilisation simple et sans accroc.L'utilisateur aura la possibilité d'accéder à un dossier Wormhole sur son bureau, où ses fichiers seront immédiatement accessibles, quel que soit l'appareil utilisé. Ils seront disponibles de manière transparente, tout comme s'ils étaient stockés localement, permettant à l'utilisateur de les consulter, de les modifier ou de les supprimer à sa convenance.

## Specification technique

### Stack thechnologique

- **Langage de programmation:** Rust
- **APIs natives:** Linux FUSE, Windows WinFsp

### Fonctionnalités

#### Intégration Native et universelle

Les dossiers Wormhole se comportent comme des dossiers normaux, intégrés de manière invisible dans le système de fichiers, que ce soit sur Windows, Linux, Mac ou Android. Cette intégration permet une expérience utilisateur fluide et transparente.

##### Stratégies de Maillage

- **Maillage simple :** Utilise un seul type de serveur, les données sont non redondantes, transmises et mises en cache à la demande.
- **Maillage complexe :** Inclut des serveurs rapides de cache, des serveurs de stockage et des serveurs utilitaires, permettant une gestion optimisée des données.

##### Modes de Sauvegarde

- **Réplication des modes RAID :** Garantit une redondance minimale pour chaque fichier.
- **Serveurs de sauvegarde optionnels :** Permet de spécifier des serveurs dédiés à la sauvegarde pour une protection accrue des données.

##### Optimisation Automatique

- **Tests de vitesse de connexion :** Évaluent les vitesses de connexion entre les noeuds pour optimiser les trajets de données.
- **Téléchargement simultané :** Permet de télécharger des données depuis plusieurs serveurs à la fois pour améliorer les performances.

##### Synchronisation Active et Balancing

- **Synchronisation Active :** Les données sont synchronisées en temps réel entre les différents noeuds, assurant une cohérence et une disponibilité immédiates.
- **Balancing :** Les charges de travail sont réparties de manière équilibrée entre les noeuds, optimisant ainsi l'utilisation des ressources.

##### Cache Interne du Service

- **Types de cache :**
  - **Disque :** Utilisé pour stocker les données fréquemment accédées.
  - **RAM :** Offre un accès ultra-rapide aux données.
  - **Spectre entre disque et RAM :** Permet de configurer la proportion de données mises en cache entre disque et RAM selon les besoins spécifiques.

##### Redondance et Résolution de Conflits

- **Redondance par réplication simple :** Le nombre de réplications est configurable pour accélérer le système.
- **Résolution de conflits :** Utilise des méthodes telles que la copie ou le choix du fichier le plus récent pour résoudre les conflits de données.
- **Affinités par pod :** Permet de configurer des affinités pour les noeuds selon les fonctions, telles que la réception des réplications ou l'optimisation de l'espace disque.

##### Téléchargement et Streaming

- **Téléchargement/Streaming depuis plusieurs noeuds :** Les données peuvent être téléchargées ou diffusées en continu depuis plusieurs noeuds simultanément, améliorant ainsi la rapidité et la fiabilité.
- **Distribution de la réplication :** Les pods serveurs distribuent les données de manière à équilibrer la charge, chaque noeud contribuant à sa part de l'upload.

##### Sécurité et Gestion des Pannes

- **Garanties de sécurité :** Mise en place de niveaux d'authentification pour sécuriser l'accès aux données.
- **Détection des pannes :** Les noeuds peuvent détecter leurs propres échecs (ex. perte de connexion Wi-Fi) et ajuster leur comportement pour maintenir l'intégrité des données.
- **Ajout/Retrait de noeuds :** Permet l'ajout ou le retrait de noeuds de manière transparente, sans compromettre l'intégrité des données.

##### Modes de Fonctionnement en Cas de Panne

- **Situation favorable :** Pas de perte de fichiers, assez de place pour recréer la redondance manquante.
- **Situation mitigée :** Pas de perte de fichiers, mais pas assez de place pour recréer la redondance.
- **Situation grave :** Perte de fichiers, nécessitant des actions correctives.

## Specification non technique

## Ideés Random (a suprimer à la fin)

### Coeur

#### seamlessness

o    intégration universelle
o      - win
o      - linux
o      - mac
o      - android

#### distribution de données

o    synchronisaion active
o    balancing

#### sécurisation de données

o    redondance
    guaranties de securité
    niveaux d'authetification

### accès

o - stockages montés
o   - linux fuse
o   - winfsp

- divers
  - api web standard (routes)
    - totale (accès et tout)
    - partielle (indique des infos comme où est un fichier, pour permettre à un client web de fetch la bonne node et éviter les restreams)
  - nfs
- conteneurs
o   - intégration docker & kubernetes

### flexibilité technique

o - cache interne du service
o   - disque
o   - RAM
o   - [tout stream <-> spèctre entre les deux <-> tout clone]

- intégrité des données
o   - redondance par réplication simple
o     - libre du nombre de réplications
o     - réplications peuvent êtres utilisées pour accélérer le système
  - methodes fancy (xor et autres)
  - guaranties logicielles
- gestion de réseau globale
  - locks
    - gérer plusieurs accès en même temps aux fichiers
      - stratégies (1 accès ever | dernier à write | accordé si majorité | etc...)
o       - résolution de conflits (copie | plus recent)
  - spécificités stratégiques
    - specification des différents stress de nodes pour une gestion plus smart
      - node de "sortie" / "préférée"
    - specifications de types de pods actifs spéciaux
o       - systèmes d'affinités par pod sur toutes les fonctions spectrales
            ex. affinité à la reception des replications / objectif espace disque...
            permet de créer des choses genre
             - pods / nodes rapides ("cache") et nodes mass storage
  - gestion de quelle node peut communiquer avec quelle node ? (faisable par pare feu mais faut que le réseau comprenne)
  - gestion de la bande passante ? (prioriser / moduler / limiter l'usage internet de certaines nodes / tout le réseau ?)

- optimisations techniques
  - optimisation automatique des réseaux suivant les vitesses d'accès et de connexion entre nodes
o   - téléchargement / streaming depuis plusieurs nodes en même temps
  - upload plus compliqué et pas forcément utile
o   - laisser les pods "servers" distribuer la réplication (pour pas qu'une node doivent uploader 6 fois le même fichier, chacun fait sa part)
  - gestion de fichiers non entiers pour opti
o      - envoyer que les modifications
o      - au dela d'un cap on ne fetch qu'en read

### Imprévu & crises

- situation favorable :
  - pas de perte de fichiers
  - assez de place pour recréer la redondance manquante
- Situation mitigée
  - pas de perte de fichiers
  - pas assez de place pour recréer la redondance et autres
- Situation grave
  - perte de fichiers

- Liste des possibilités
o   - rebalance (du mieux possible)
o   - readonly (freeze)
o   - coupure d'accès
- Gestion de l'attente
  - tous les cas ou c'est qu'une déco, plus destiné particulier qu'entreprise j'imagine
    - timer en cas de situation favorable (avant de reshape toute la structure)
    - methodes comme les autres pour les deux autres cas
- procédure de réinsertion
o  - rebalance automatique quand possible
- check et gestion des conflits
o - detection de sa propre failure (plus de wifi)
- eviter de panic quand toutes les nodes du réseau ne répondent plus

### fonctionalités additionnelles

o - ajout / retrait seamless de nodes (quand ne brise pas l'intégrité des données)

o - pods passifs (portals / clients)

- pod mirror-portal (voir avec J)
    "pod mirror qui représente un pod qui est là par intermittence,
    regulant les concurrence d'actions sur ce pod"
- configurations en niveaux
o   - cluster (réseau global)
  - node (?)
o       - pod (affinités et infos locales)
o         - par dossiers / fichiers
            ex. keep

### sécurité

- chiffrage
  - des données dans le réseau
