sécurité en cas de défaillance des données :

Redondance

théorie générale de la redondance x
les disques doivent être séparés en x groupes de taille totale similaire
le stockage utilisable pour le vortex est dicté par le plus petit groupe
pour l'utilisateur, cette taille est divisée par le nombre de redondances

duplication des fichiers sur différents nœuds
  + quelques contraintes 
    la taille inégale des disques peut être gérée assez facilement dans la plupart des cas
  + répartition intelligente des fichiers là où ils sont le plus utilisés => remplace librement l'utilisation du cache tout en étant utile
  + téléchargement parallèle multi-nœuds (+ vitesse de lecture)
  + Grande disponibilité en cas de défaillance
      permet autant de défaillances de nœuds que de répliques
      pas d'effet si suffisamment de stockage et de nœuds pour continuer à répliquer
      peut toujours fonctionner (moins sécurisé mais fonctionnel) s'il n'y a pas assez d'espace pour continuer la réplication
  ~+ évolue tous les (x) nœuds ajoutés
  ~+ téléchargement parallèle multi-nœuds (+ vitesse d'écriture) (téléchargement d'une partie du fichier sur chaque nœud (plus rapide) mais peu sûr lors du réassemblage du fichier entre les nœuds)
  - stress du réseau lors de l'écriture
  - grande utilisation de l'espace de stockage (50% d'efficacité sur 1 rep)

méthodes fantaisistes

méthodes fantaisistesParité (comme Raid 4-5)Diviser les fichiers en deux et stocker la parité sur le troisième noeud  + efficace sur l'espace de stockage (66%)  ~ 3 noeuds et configuration plus contraignante
      appliquer la théorie générale avec x=3, mais l'espace utilisateur n'est divisé que par 1,66, ce qui permet à un noeud de tomber en panne      ne peut évoluer qu'avec l'ajout de 3 noeuds
  ~ Ok uptime pendant la panne      configuration à 3 noeuds :        // TODO 
- pas utilisé comme cachepar définition, au repos, un seul fichier devrait être divisé avec seulement la moitié sur un disque  
- équitable mais stress de traitement existant sur l'écriture (xor)