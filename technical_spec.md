# Spécification Technique
Comme expliqué dans le contexte du projet, Wormhole est une solution de stockage décentralisé de données.
Cette partie du document propose une rapide explication de ce qu'est la décentralisation, et de comment cette méthode se compare aux autres.
Le détail technique des fonctions proposées par le projet ainsi que sa stack technique sera ensuite abordé.

## La décentralisation (contexte - définition - utilité)
Aujoud'hui, petites comme grandes entreprises ont de grands besoins en terme de stockage de données :
- Données internes
  - Documents de l'entreprise (cloud interne pour les employés)
  - Données de travail
    - Assets pour un studio de jeu vidéo
    - Datasets scientifiques pour un laboratoire
    - Training sets pour studios d'intelligence artificielle
    - Big Data
    - ... toute donnée servant directement l'entreprise
  - Données sensibles
    - Comptes, devis et factures de l'entreprise (données légales)
    - Données en rapport avec un client
- Données utilisés par un service logiciel proposé par l'entreprise
  - Musiques pour une application comme Spotify/Deezer
  - Vidéos pour une application comme Youtube/TikTok
  - Diverses données stockées pour un service comme OneDrive/Google Drive

Tous ces usages ne sont que des exemples mais représentent bien les besoins qu'ont les entreprises correctement implantés dans l'ère informatique.
Cependant, ce besoin est vite limité par un plafond de verre.
En effet, on ne peut pas concentrer une infinité de ressources dans un seul serveur.
Centraliser la donnée sur une seule machine poserait aussi un problème d'intégrité des données en cas de panne.

**Très vite arrive la nécéssité de multiplier le nombre de machines pour répondre au moins à certaines des exigences suivantes :**
- Besoin de capacité massive de stockage (plus de place)
- Besoin de plus de puissance (servir les données plus vite)
- Gestion de crise (résister à une panne sans interruption de service ni perte de données)
- Faciliter l'accès à tous les sites géographiques de l'entreprise

Multiplier le nombre de machines pour un même service s'appelle de la décentralisation, par opposition à la centralisation, restreinte à une entité.   
Face à ce besoin incontournable, les entreprises ont peu de solutions :
- **Utiliser un fournisseur cloud externe**   
  C'est la solution la plus simple.   
  Elle est cependant couteuse et l'entreprise n'est plus souveraine de ses données.   
  Cela la rend impossible dans certains cas (données sensibles, données utilisées par un service logiciel ou besoin spécifique)   
  *A noter que les services cloud utilisent justement la décentralisation pour sécuriser les données*
- **Semi-centralisation (manuelle)**   
  Solution consistant à garder le plus possible une entitée (serveur / salle serveur) principale, et d'en prévoir une seconde hors ligne sur laquelle on sauvegarde régulièrement.   
  En cas de panne, on connecte la seconde entité en remplacement. On l'utilisera aussi pour remettre les données sur l'entité principale une fois celle ci en état de marche.   
  Cette stratégie est plus utilisée sur les infrastructures à échelle datacenter. Peu accessible par les entreprises moyennes.
- **Décentralisation (manuelle)**   
  La solution ultime, répondant à tous les besoins dont nous avons parlé.   
  Cependant il n'existe pas de moyen universel pour mettre en place cette solution. **C'est à cela que Wormhole répond,** en proposant un outil simple, ouvert et universel.

Wormhole se veut être le Kubernetes de l'espace disque.

## Résumé de Wormhole
(expliquer que c'est un système de fichiers donc qu'on le voit comme un disque / dossier normal)


## specification
lister globalement tout, presque comme une doc de conf, mais en moins rude