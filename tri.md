# Innovation Track

## Context



## Specification technique

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


### accès :
o - stockages montés
o   - linux fuse
o   - winfsp
 - divers
   - api web standard (routes)
x     - totale (accès et tout)
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
x   - guaranties logicielles
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