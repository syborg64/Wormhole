# Spécification Technique
Comme expliqué dans le contexte du projet, Wormhole est une solution de **stockage décentralisé de données**.   
Cette partie du document propose une rapide explication de ce qu'est la décentralisation, et de comment cette méthode se compare aux autres.   
Le détail technique des fonctions proposées par le projet ainsi que sa stack technique sera ensuite abordé.

## La décentralisation (contexte - définition - utilité)
Aujoud'hui, petites comme grandes entreprises ont de grands besoins en terme de stockage de données :
- **Données internes**
  - Documents de l'entreprise (cloud interne pour les employés)
  - Données de travail   
    > Assets pour un studio de jeu vidéo   
    > Datasets scientifiques pour un laboratoire   
    > Training sets pour studios d'intelligence artificielle   
    > Big Data   
    > ... toute donnée servant directement l'entreprise   
  - Données sensibles
    > Comptes, devis et factures de l'entreprise (données légales)   
    > Données en rapport avec un client   
- **Données utilisés par un service logiciel proposé par l'entreprise**
  > Musiques pour une application comme Spotify/Deezer   
  > Vidéos pour une application comme Youtube/TikTok   
  > Diverses données stockées pour un service comme OneDrive/Google Drive   

Tous ces usages ne sont que des exemples mais représentent bien les besoins qu'ont les entreprises correctement implantés dans l'ère informatique.   
**Cependant, ce besoin est vite limité par un plafond de verre.**   
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
  > C'est la solution la plus simple.   
  > Elle est cependant couteuse et l'entreprise n'est plus souveraine de ses données.   
  > Cela la rend impossible dans certains cas (données sensibles, données utilisées par un service logiciel ou besoin spécifique)   
  > *A noter que les services cloud utilisent justement la décentralisation pour sécuriser les données*
- **Semi-centralisation (manuelle)**   
  > Solution consistant à garder le plus possible une entitée centralisée (serveur / salle serveur) principale, et d'en prévoir une seconde hors ligne sur laquelle on sauvegarde régulièrement.   
  > En cas de panne, on connecte la seconde entité en remplacement. On l'utilisera aussi pour remettre les données sur l'entité principale une fois celle ci en état de marche.   
  > Cette stratégie est plus utilisée sur les infrastructures à échelle datacenter. Peu accessible par les entreprises moyennes.   
  > Elle induit aussi une possible interruption de service.
- **Décentralisation (manuelle)**   
  > **La solution ultime**, répondant à tous les besoins dont nous avons parlé.   
  > **Cependant il n'existe pas de moyen universel pour mettre en place cette solution. C'est à cela que Wormhole répond,** en proposant un outil simple, ouvert et universel.

> [!TIP] Wormhole se veut être le Kubernetes de l'espace disque.

## Notre solution : Wormhole
**Wormhole offre une solution simple et déclarative pour la création d'infrastructures décentralisées simples comme avancées.**   
Wormhole créé un système de fichiers décentralisé entre toutes les machines ajoutés au réseau.   
Une fois monté, ce système de fichier, intégré nativement, ne diffère pas des autres fichiers de la machine.
> [!NOTE] Pour un utilisateur, impossible de différencier un dossier de fichiers locaux et un dossier Wormhole.   
> Il en va de même les logiciels et les applications, les fichiers se comportant comme des fichiers locaux normaux, aucune adaptation n'est nécéssaire.

### Pour les entreprises :
Adapté aux besoins de grande échelle, Wormhole permet de monter en un claquement de doigt une infrastructure puissante :
- **Massive**, libérée de la centralisation sur un serveur, la croissance n'a pas de limite.
- **Performante**, tirant parti de toute la puissance mise à disposition de manière optimisée, évitant la consomation inutile.
- **Sécurisée** contre les pertes de données (même en cas de panne).
- **Sans interruption de service**, même en cas de panne, même lors de modification du réseau.
- **Flexible**, avec modification facile de l'infrastructure sans interruption de service.
- **Native**, sans besoin d'adapter les applications et services déjà présents.
- **Adaptée** à toutes les échelles, du petit réseau local d'une startup jusqu'aux grandes infrastructures internationales.

> [!IMPORTANT] La configuration simple, claire et déclarative permet d'éviter l'erreur humaine.   
> Une fois lancé, l'expérience sera fluide et fiable pour tous les services.
> Le réseau peut être modifié, des machines ajoutées ou retirées sans interrompre le service.   
> L'entreprise peut facilement définir sa gestion de sécurité pour la concervation des données, ainsi que les plans de gestion de crise pour résister même aux pannes les plus massives.
<br>

> [!TIP] Evolutif / Scalable
> La nature adaptive de Wormhole le rend ouvert à des utilisations variées.   
> **Léger**, ne demande pas de configuration minimale puissante.   
> **Optimisé**, il tirera parti des serveurs les plus capables.   

#### Exemples d'utilisations (User Stories) :

> ➕**Startup / PME dans la cybersécurité**   
> Petite équipe, n'a pas de pôle DSI pour gérer de l'infrastructure.   
> N'utilise pas de cloud externe afin de garder la souveraineté de ses données.   
> Héberge ses données sur ses quelques (ex. 3) petits serveurs NAS.
> - Souhaite simplifier l'organisation de ses données (actuellement éparpillées sur les différents NAS)
> - Souhaite assurer l'intégrité de ses données en cas de panne
> - N'a pas de temps ni d'équipe à consacrer à cette gestion des données (organisation, sauvegarde, accès...)
> - Aimerait une solution qui pourra croitre avec l'entreprise
>
> **Solution Wormhole :**
> - Les machines d'un réseau sont "fusionnées". Pour l'utilisateur final, il n'y a qu'une racine (/) peu importe le nombre de machines individuelles. Libre à lui de créer les dossiers et l'organisation qu'il souhaite.
> - La configuration d'intégrité est très complète, elle permet d'anticiper et de réagir aux imprévus. Voici quelques exemples :
>   - L'option de redondance stocke la quantité demandée de copies d'un même fichier sur plusieurs machines. Le plus le nombre copies, le moins le risque de perte !
>   - Les options gestion de crise permettent prévoir la posture à adopter si trop de machinent tombent pour continuer le fonctionnement normal.
> - La création d'un réseau est faisable rapidement même par un débutant, et ne demande pas de gestion une fois en place.
> - La modification d'un réseau ne nécéssite pas sa suppression, il s'équilibre automatiquement lors de l'ajout ou du retrait d'une machine.
>   Il est donc facilement portable sur une infrastructure croissante.
<br>
___

> ➕**Laboratoire**   
> Equipe spécialisée, a des serveurs et machine puissantes, mais ce n'est pas le coeur de métier.   
> Procède à des simulations et analyses, générant des flux très importants de données.   
> N'utilise pas de cloud externe, incompatible avec ses besoins de performance.   
> Détient des machines puissantes mais spécialisées (Ordinateurs pour simulation GPU, Ordinateurs pour analyse CPU, serveurs de stockage massifs).
> - A de grands besoins de performances.
> - Souhaiterait que plusieurs machines distinctes puissent analyser un même set de données.
> - Les données sont générées, analysées et supprimées au jour le jour, la perte en cas de panne n'est pas un problème.
> - A des besoins très changeants (oscille régulièrement entre quelques Go et quelques dixaines de To) et aimerait pouvoir allouer ses ressources au jour le jour.
>
> **Solution Wormhole :**
> - Stocke intelligemment les données là où elles sont le plus demandées. Propose un système de cache pour accélérer le système.
> - Chaque machine du réseau a en effet le même set de données.
> - La configuration permet totalement d'optimiser le réseau pour la vitesse et non pour l'intégrité au long terme.
> - La rapidité et simplicité de mise en place d'un réseau permet totalement de monter, utiliser et supprimer un réseau pour une seule utilisation.
>   De plus, il suffit de garder le fichier de configuration sous la main pour recréer le réseau en une commande.
<br>
___

> ➕**Service web**   
> Entreprise récente venant d'exploser ! Ce nouveau réseau social permet de partager non pas des photos mais des scans 3D !
> Le réseau est atypique mais possède déjà 10.000 utilisateurs réguliers ! Stocker tous ces posts pèse lourd !
> - A un besoin grandissant de place.
> - A un besoin contrasté de performance. Les ressources devraient êtres priorisées pour les posts en tendances plutôt que les posts anciens et rarement vus.
> - A besoin d'un service ininterrompu même en cas de panne.
> - A des exigences d'intégrité autour du minimum légal (autour de 3 copies)
>
> **Solution Wormhole :**
> - Utilise toutes les ressources qui lui sont offertes, et en permet un ajout facile.
> - La configuration des systèmes de cache et d'affinités permet de distinguer les serveurs rapides (SSD) et massifs (HDD) et d'utiliser au mieux leur potentiel.
> - Le réseau maintenant installé sur une telle quantité de serveurs, la redondance, l'équilibrage automatique et la gestion de crise rendent une interruption de service ou une perte de données virtuellement impossibles.

<br>
Une fois le système mis en place, tout fonctionne automatiquement, garantissant une utilisation simple et sans accroc.   
La configuration par fichier est réutilisable et partageable. Sa clareté la rend facile à comprendre et maintenir même des années après sa mise en place.
La plasticité du réseau le rend fiable, adaptable et modifiable sans mesures compliquées.

### Pour les particuliers
La nature **flexible** de Wormhole lui permet un usage pratique même chez les particuliers.   
Marre de chercher vos documents, photos et projets entre votre NAS, votre ordinateur fixe et votre ordinateur portable?   
Montez en quelques minutes un réseau Wormhole, et vos différents appareils ne font plus qu'un. Vos données sont disponibles sur tous comme si elles y étaient !   
> [!IMPORTANT] Une fois installé, on oublie très vite la présence de Wormhole.   
> Et pourtant, l'enfer de chercher ses données sur différents appareils, les synchroniser ou les sauvegarder est maintenant de l'histoire ancienne.   
> Wormhole fait tout pour vous 😎   
> On vous a volé votre pc portable ? **Vous n'avez pas perdu vos données.**   
> Votre NAS déraille ? **Vous n'avez pas perdu vos données.**   
> Votre ordinateur fixe brule ?! **Vous n'avez pas perdu vos données !**   
> Vous avez un nouvel appareil ? **Une commande, et tout est géré.**

___

## specification
partie Arthur