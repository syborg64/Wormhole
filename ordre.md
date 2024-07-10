(les blocs c'est pas des sprints, et l'odre est changeable dans une certaine mesure)


1 semaine = 1 jour par personne = 4 JeH
1 sprint = 6 semaines = 6 * 4 = 24 JeH
EIP = 6 sprints = 24 * 6 = 144 JeH
1 personne travaille 36 jours durant l'EIP

Milestones:
    sprint 1:
        MVP
            - connect arbitrarily to peers
            - add & remove pods
            - create, modify, read, and delete files off the network
            - move files from pods
            - natively integrated on linux
    sprint 2
        full metadata
            - local awareness of remote file status
            - QoL (import)
        config
            - global config
            - per pod config
            - config template ?
    sprint 3
        replication & cache
            - local storage cache
            - ownership
            - move operations respecting replication count
    sprint 4
        failure conditions
            - identify reliable peers
            - recovery systems
            - conflicts
            - data loss reporting
            - resync
    sprint 5
        performance improvements (chopping bloc)
            - benchmarking
            - major refactoring
            - search for hot slow code
        system balancing
            - system-based cache pruning
            - track file acess and usage
            - automatically move files to hotest pod
    sprint 6
        safety
            - software guarantees
            - perfect error handling
        authentification
            - key-pair based authentification
            - ssh based

Borne:
    sprint 1:
        MVP
            - connection automatique aux autres nodes
            - ajout & retir des pods
            - création, modification, lecture et suppression de fichier sur le réseau
            - déplacer des fichiers entre les pods
            - intégration native linux
    sprint 2
        métadonnée complète
            - information locale persistante des fichiers distants
            - fonctionnalités d'aisance (importation)
        configuration
            - configuration par réseau
            - configuration par pod
            - template de configuration
    sprint 3
        replication et cache
            - cache en stockage local
            - possession
            - replication N
            - déplacement de fichiers répliqués
            - Affinité par pod pour la redondance
        mise en place de la gestion de pannes
            - identifier l'état de santé du réseau
            - notification et log
    sprint 4
        intégration d'autres API
            - fonctionalités équivalentes avec la version FUSE
            - integration windows
            - Intégration Docker / Kubernetes

        stratégies de récupérations            
            - compensation / rééquilibrage
            - conciliation de conflicts
            - systèmes de récupération
            - resynchronisation
    sprint 5
        amélioration des performances (leste)
            - tests de performances
            - refactorations majeures 
            - identification de code hot & lent
        équilibrage du système
            - nettoyage du cache automatisé
            - track les accès et utilisations de fichier
            - déplacement automatique des fichiers vers le pod le plus demandant
    sprint 6
        sécurité
            - gestion d'erreur intégrale et verbeuse
        authentification
            - authentification TLS/paire de clé
            - par ssh

### bloc A - MVP
Reproduction++ au propre du proto (MVP)
    integration linux
    cmd et configuration de base
    conception et mvp architecture du système de base (décentralisée, savoir où est quel fichier, y accéder, etc)

    Rester léger sur l'intégration de fuse (nécéssaire pour open read write uniquement. metadonnées templates si non nécéssaires)

    ajout seamless de node vides
    retrait seamless de node (voulu, en bonnes conditions)
        exporter toutes ses données sur les autres
    
    les objectifs sont pensés en conditions idéales :
    pas de gestion de conflits (garde le plus récent)
    pas de conf par pod
    pas de redondance


### bloc A+ - consolidation
continuer l'intégration de fuse (gérer les vraies métadonnées)

optionnel mais de bon gout :
ajout de nodes avec un dossier de mount non vide de base :
    si son mount contient des fichiers, les ajouter au cluster
    Vérifications nécéssaires avant de procéder (espace nécéssaire, redondance incluse)
    Gestion du conflit simple sur cette intégration (donné lors de la commande)
        Garder le fichier du cluster
        Garder le fichier de la nouvelle node
        Garder le fichier modifié le plus recemment


### bloc B
développement de la configuration globale et par pod

config pod
    espace disque max
    espace disque objectif (EDO)
    cache et données locales (sur disque)
        [tout stream <-> entre les deux <-> tout clone]
        local, pas utilisé par le cluster

redondance
    pas utilisée pour accélérer le systeme pour l'instant
    l'équilibrer avec les EDO des pods

début de tests avec une gestion de crise dans la situation "favorable"
    coté cluster : rebalance (recréer la redondance manquante)
    coté node : à voir mais faire simple 
    commencer la création d'une procédure de ré-insertion
        avec gestion de conflits simple
            - si un seul des deux à modifié -> version modifié sur tout le monde
            - si les deux ont modifié -> version du cluster
    penser et anticiper une manière de prévenir l'utilisateur (on vise un publique serveur)

### bloc B+ - consolidation
Approfondir le début de gestion de crise (cas mitigé) :
Faire les vérifications nécéssaire pour déterminer la situation actuelle
    - Favorable (gérée sur bloc B)
    - Mitigée   (gérée ici)
    - Critique  (pas encore gérée)
Offrir plus de configuration pour fit la situation mitigée (et ça servira pour favorable aussi tant qu'à faire)
    - seamless unsafe : Autoriser l'écriture le plus longtemps possible, quitte à supprimer des replications si besoin d'espace
        > (option mais quali) garder en tête la capacité de stockage de la node disparue pour éviter de manquer de stockage si elle est rebranché et qu'on tente de réatteindre l'objectif de réplications
    - readonly flexible (un fichier déjà open peut être write)
    - readonly strict
    - freeze total (idk why but I mean you can)

penser (sans prétention) aux cas de coupures radicales à des moments cruciaux

optimiser le stress causé par la redondance 
    voir main_conf.md ligne 60
    faire en sorte que chaque node fasse sa part plutôt qu'une upload beaucoup de fois


##############################
blocs manquants (eh il est tard)
##############################



# blocs à penser

### experience utilisateur
assistants / templates de configuration
s'assurer que la cli permet rapidement et facilement les choses
doc utilisateur accessible
Toujours penser à un max de configuration par défaut

objectif :
Le réseau le plus simple (raid 0) devrait avoir un fichier de configuration quasi vide (voir vide) et pouvoir être créé très rapidement
Les configurations les plus mainstream pourraient êtres proposées
Les configurations avancées devraient rester claires (dans la mesure du possible)
La cli devrait être aussi claire et complète que l'est le compilo rust
    - Detection le plus tôt possible de problèmes, vitaux ou non (cas impossible - bottleneck/stress - inutile)
    - communiqués en erreurs, warnings et conseils clairs

outils d'analyses
    très large et vague, on en a pas parlé, donc ça passe très probablement en bonus
    mais ça pourrait être intéréssant et de grande valeur pour un sys admin
    idées entre autres
        - temps de réponse d'une node, de node a node...
        - routes opérant beaucoup de transits
        - nodes sous stress
        - latence ou lenteur quelque part dans le réseau
        ...


### outils dev (nous)
Il faudra penser à quelque chose pour tester notre solution
Ca pourrait possiblement se faire littéralement avec kubernetes en spawnant un cluster local, et ça serait deux en 1

### bloc opti
(en vrai c'est probablement à répartir un peu partout mais je le note ici)
facile :
 - cache local (garde les données selon la conf du pod. Le réseau global ne s'en sert pas)

moyen / difficile :
 - optimiser l'endroit de stockage des fichiers (classé par difficulté)
  1 les endroits qui l'utilisent souvent
  2 utiliser les redondances de la même manière
  3 traquer et utiliser le cache local de pod et l'utiliser aussi^1
  4 prendre en compte le réseau^2

bien réfléchir à la manière d'indexion des fichiers
    car une simple hashmap ça pourrait pleurer un peu si on tape quelques millions de fichiers
    faut faire des recherches, il existe pleins de structures de données, y compris décentralisées

## blocs étendue de compatibilités

### bloc docker / kubernetes
peut être utilisé par nous mêmes pour du test
ne devrait pas être trop compliqué

### bloc windows
sera possiblement long, si on ne veut pas un truc bancal

### bloc android
(moi je trouve que si on a de l'énergie pour ce bloc, on devrait la replacer par une api web, qui ouvre de vraies portes)
(après je ne suis pas sur qu'on atteigne ce bloc, de manière globale)



_____________________________________________________________________________________

^1 - lui je ne l'avais pas noté et j'ai eu l'idée ce soir. sera probablement pas dans les sprints du coup, mais l'idée est smart
    j'entendais parler de ça parce qu'en entreprise, quand on deploie une maj windows aux 3000 pc d'un batiment, seuls 2-3 pull depuis les serveurs microsoft.
    Et après tous les autres se pull entre eux sur le réseau local par effet boule de neige. ça économise de la bande passante pour tout le monde
    le process est intéréssant fait sur un réseau local comme ça.
    Et ça rejoint notre idée (qui elle est prévu) de répartir la charge de réplication coté des nodes

^2 - on a dit qu'on ne faisait pas ça et j'accepte la volonté du groupe (surtout en vue du peu de jours sur le projet)
mais si on a du temps ou qu'on doit reprioriser
c'est dommage que tout un réseau se vautre si une seule node bottleneck