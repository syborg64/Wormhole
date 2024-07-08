(les blocs c'est pas des sprints)

### bloc 1
Reproduction++ au propre du proto (MVP)
    integration linux
    cmd et configuration de base
    architecture du système de base (décentralisée, savoir où est quel fichier, y accéder, etc)

    conditions idéales
    pas de gestion de conflits (garde le plus récent)
    pas de conf par pod
    pas de redondance
    ajout seamless de node
        si son mount contient des fichiers, les ajouter au réseau (indexer)
    retrait seamless de node (voulu, en bonnes conditions)
        exporter toutes ses données sur les autres


### bloc 2
développement de la configuration globale et par pod
    redondance
        pas forcément utilisée pour accélérer le systeme pour l'instant
    cache et données locales [tout stream <-> spèctre entre les deux <-> tout clone]

début de tests avec une gestion de crise dans la situation "favorable"
    recréer la redondance manquante
    commencer la création d'une procédure de ré-insertion
        avec gestion de conflits simple
            - si un seul des deux à modifié -> version modifié sur tout le monde
            - si les deux ont modifié -> version du réseau

config pod de base
    espace disque
    affinité pour redondance
