# Innovation Track

## Context



## Specification technique

## Specification non technique

## Ideés Random (a suprimer à la fin)


### Coeur
  #### seamlessness
    intégration universelle
      - win
      - linux
      - mac
  #### distribution de données
    synchronisaion active
    balancing

  #### sécurisation de données
    redondance
    guaranties de securité
    niveaux d'authetification


### accès :
 - stockages montés
   - linux fuse
   - winfsp
 - divers
   - api web standard (routes)
     - totale (accès et tout)
     - partielle (indique des infos comme où est un fichier, pour permettre à un client web de fetch la bonne node et éviter les restreams)

 - conteneurs
   - intégration docker & kubernetes


### flexibilité technique

 - cache
   - [tout stream <-> spèctre entre les deux <-> tout clone]
 - intégrité des données
   - redondance par réplication simple
     - libre du nombre de réplications
     - réplications peuvent êtres utilisées pour accélérer le système
    - methodes fancy (xor et autres)
   - guaranties logicielles
 - gestion de réseau globale
   - locks
     - gérer plusieurs accès en même temps aux fichiers
       - stratégies (1 accès ever | dernier à write | accordé si majorité | etc...)
       - résolution de conflits
   - spécificités stratégiques
     - specification des différents stress de nodes pour une gestion plus smart
       - node de "sortie" / "préférée"
     - specifications de types de pods actifs spéciaux
       - systèmes d'affinités par pod sur toutes les fonctions spectrales
            ex. affinité à la reception des replications / objectif espace disque...
            permet de créer des choses genre
             - pods / nodes rapides ("cache") et nodes mass storage
   - gestion de quelle node peut communiquer avec quelle node ? (faisable par pare feu mais faut que le réseau comprenne)
   - gestion de la bande passante ? (prioriser / moduler / limiter l'usage internet de certaines nodes / tout le réseau ?)

 - optimisations techniques
   - optimisation automatique des réseaux suivant les vitesses d'accès et de connexion entre nodes
   - téléchargement / streaming / upload depuis plusieurs nodes en même temps
     - (upload plus compliqué et pas forcément utile)
   -

### gestion de crise

- situation favorable :
  - pas de perte de fichiers
  - assez de place pour recréer la redondance manquante
- Situation mitigée
  - pas de perte de fichiers
  - pas assez de place pour recréer la redondance et autres
- Situation grave
  - perte de fichiers

 - Liste des possibilités
   - rebalance (surtout pour les situations favorables)
   - readonly (freeze)
   - coupure d'accès
 - Gestion de l'attente
   - tous les cas ou c'est qu'une déco, plus destiné particulier qu'entreprise j'imagine
 - procédure de réinsertion
  - rebalance automatique quand possible
  - check et gestion des conflits
 - detection de sa propre failure (plus de wifi)
   - eviter de panic quand toutes les nodes du réseau ne répondent plus

### fonctionalités additionnelles

 - pods passifs (portals / clients)
 - pod mirror-portal (voir avec J)
    "pod mirror qui représente un pod qui est là par intermittence,
    regulant les concurrence d'actions sur ce pod"
 - configurations en niveaux
   - cluster (réseau global)
     - node (?)
       - pod (affinités et infos locales)
         - par dossiers / fichiers
           ex. keep
 -

### sécurité

- chiffrage
  - des données dans le réseau