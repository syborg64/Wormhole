<style>
  blockquote {
  border-left: .25em solid #888;
  border-left-color: lightblue;
}
</style>

# Wormhole

## Contexte

Le projet Wormhole est né de la nécessité de simplifier l'accès et la gestion des données entre plusieurs serveurs. Actuellement, les entreprises sont confrontées à plusieurs défis liés à la centralisation ou à la décentralisation des données. Les solutions centralisées présentent des risques de sécurité, des limitations d'accès et un poids accru sur l'infrastructure. En revanche, les infrastructures décentralisées sont difficiles à mettre en place et manquent de solutions universelles.  

Wormhole se positionne comme une solution technique innovante visant à offrir un moyen simple et universel de construire une infrastructure décentralisée adaptée, sécurisée et souveraine pour ses données. Le projet vise à répondre aux besoins de sécurité, de flexibilité et de simplicité de configuration pour les entreprises et les particuliers.  

## Spécification Technique
Comme expliqué dans le contexte du projet, Wormhole est une solution de **stockage décentralisé de données**, c'est à dire un stockage sur plusieurs machines distinctes et non sur une seule machine centrale.   
Cette partie du document propose une rapide explication de ce qu'est la décentralisation, et de comment cette méthode se compare aux autres.   
Le détail technique des fonctions proposées par le projet ainsi que sa stack technique sera ensuite abordé.  

### La décentralisation (contexte - définition - utilité)
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
    > Données privées en rapport avec un client   
- **Données utilisés par un service logiciel proposé par l'entreprise**
  > Musiques pour une application comme Spotify/Deezer   
  > Vidéos pour une application comme Youtube/TikTok   
  > Diverses données stockées pour un service comme OneDrive/Google Drive   

Tous ces usages ne sont que des exemples mais représentent bien les besoins qu'ont les entreprises correctement implantés dans l'ère informatique.   
**Cependant, ce besoin est vite limité par une limite physique.**   
En effet, on ne peut pas concentrer une infinité de ressources dans un seul serveur.   
Centraliser la donnée sur une seule machine poserait aussi un problème d'intégrité des données en cas de panne.   

**Très vite arrive la nécéssité d'augmenter le nombre de machines pour répondre au moins à certaines des exigences suivantes :**
- Besoin de capacité massive de stockage (plus de place)
- Besoin de plus de puissance (servir les données plus vite)
- Fiabilité / Gestion de crise
  - Résister sans effort aux pannes mineures
  - Suivre sa politique de PCA/PCI ([Plan de Continuité d'activité Informatique](https://fr.wikipedia.org/wiki/Plan_de_continuit%C3%A9_d%27activit%C3%A9_(informatique))) en cas d'incidant majeur
- Faciliter l'accès pour tous les sites géographiques de l'entreprise

> [!TIP] Plan de Continuité d'Activité / Informatique
> La **PCA/PCI** est une pratique courante pour les entreprises dépendantes de services informatiques.   
> Généralement mise en place par la direction informatique ainsi que les coeurs de métiers concernés, elle prend la forme d'une procédure claire de réaction aux incidents graves les plus probables.   
> Wormhole n'écrit pas ce plan pour l'entreprise, mais dispose des paramètres nécéssaire pour respecter des procédures définies à l'avance.   
> Plus d'informations : [Wikipédia - Plan de continuité d'activité (informatique)](https://fr.wikipedia.org/wiki/Plan_de_continuit%C3%A9_d%27activit%C3%A9_(informatique))

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

### Notre solution : Wormhole
**Wormhole offre une solution simple et déclarative pour la création d'infrastructures décentralisées simples comme avancées.**   
Wormhole créé un système de fichiers décentralisé entre toutes les machines ajoutés au cluster (groupe de machines Wormhole).   
Une fois monté, ce système de fichier, intégré nativement, ne diffère pas des autres fichiers de la machine.  
> [!NOTE] Pour un utilisateur, il n'y a aucune différence entre un dossier de fichiers locaux et un dossier Wormhole.   
> Il en va de même les logiciels et les applications, les fichiers se comportant comme des fichiers locaux normaux, aucune adaptation n'est nécéssaire.  

#### Pour les entreprises :
Adapté aux besoins de grande échelle, Wormhole permet de monter en un claquement de doigt une infrastructure puissante :
- **Massive**, libérée de la centralisation sur un serveur, la croissance n'a pas de limite.  
- **Performante**, tirant parti de toute la puissance mise à disposition de manière optimisée, évitant la consomation inutile.  
- **Sécurisée** contre les pertes de données (même en cas de panne).  
- **Sans interruption de service**, même en cas de panne, même lors de modification du réseau.  
- **Flexible**, avec modification facile de l'infrastructure sans interruption de service.  
- **Native**, sans besoin d'adapter les applications et services déjà présents.  
- **Adaptée** à toutes les échelles, du petit réseau local d'une startup jusqu'aux grandes infrastructures internationales.  

> [!IMPORTANT] La configuration simple, claire et déclarative permet d'éviter l'erreur humaine.   
> Une fois lancée, l'expérience sera fluide et fiable pour tous les services.  
> Le réseau peut être modifié, des machines ajoutées ou retirées sans interrompre le service.   
> L'entreprise peut facilement définir sa gestion de sécurité pour la concervation des données, ainsi que ses [plans de continuité d'activité informatique](https://fr.wikipedia.org/wiki/Plan_de_continuit%C3%A9_d%27activit%C3%A9_(informatique)) pour résister aux incidents mineurs comme majeurs.  

> [!TIP] Evolutif / Scalable
> La nature adaptive de Wormhole le rend ouvert à des utilisations variées.   
> **Léger**, ne demande pas de configuration minimale puissante.   
> **Optimisé**, il tirera parti des serveurs les plus capables.   

### Exemples d'utilisations (User Stories) :
> [!WARNING] Vocabulaire utilisé :
> - **Cluster** : Désigne le groupe de machines (nodes) connectées sur un même réseau Wormhole.   
> - **Node** : Désigne une machine individuelle faisant partie d'un réseau.   
> - **Pod** : Désigne la partie d'une node dédiée à un cluster (un même node pouvant participer à plusieurs clusters indépendants).   

> ➕**Startup / PME dans la cybersécurité**   
> Petite équipe, n'a pas de pôle DSI pour gérer de l'infrastructure.   
> N'utilise pas de cloud externe afin de garder la souveraineté de ses données.   
> Héberge ses données sur ses quelques (ex. 3) petits serveurs NAS.  
> - Souhaite simplifier l'organisation de ses données. (Actuellement éparpillées sur les différents NAS).
> - Souhaite assurer l'intégrité de ses données en cas de panne.
> - N'a pas de temps ni d'équipe à consacrer à cette gestion des données. (Organisation, sauvegarde, accès...)
> - Aimerait une solution qui pourra croitre avec l'entreprise.
>
> **Solution Wormhole :**
> - Les machines d'un cluster sont "fusionnées". Pour l'utilisateur final, il n'y a qu'une racine (/) peu importe le nombre de machines individuelles. Libre à lui de créer les dossiers et l'organisation qu'il souhaite.  
> - La configuration des stratégies d'intégrité est très complète, elle permet d'anticiper et de réagir aux imprévus. Voici quelques exemples :
>   - L'option de redondance stocke la quantité demandée de copies d'un même fichier sur plusieurs machines. Plus il y a de copies, moins le risque de perte est important en cas de panne.  
>   - Les options gestion de crise ([PCI](https://fr.wikipedia.org/wiki/Plan_de_continuit%C3%A9_d%27activit%C3%A9_(informatique))) permettent prévoir la posture à adopter si trop de machines tombent pour continuer le fonctionnement normal.  
> - La création d'un cluster est faisable rapidement même par un débutant, et ne demande pas de gestion une fois en place.  
> - La modification d'un cluster ne nécéssite pas sa suppression ni même d'arrêt temporaire, il s'équilibre automatiquement lors de l'ajout ou du retrait d'une node.  
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
> - Chaque machine du cluster a en effet le même set de données.  
> - La configuration permet totalement d'optimiser le cluster pour la vitesse et non pour l'intégrité au long terme.  
> - La rapidité et simplicité de mise en place d'un cluster permet totalement de monter, utiliser et supprimer un cluster pour une seule utilisation.  
>   De plus, il suffit de garder le fichier de configuration sous la main pour recréer le cluster en une commande.  
<br>
___

> ➕**Service web**   
> Entreprise récente venant d'exploser ! Ce nouveau réseau social permet de partager non pas des photos mais des scans 3D !
> Le réseau est atypique mais possède déjà 10.000 utilisateurs réguliers ! Stocker tous ces posts pèse lourd !
> - A un besoin grandissant de place.  
> - A un besoin contrasté de performance. Les ressources devraient êtres priorisées pour les posts en tendances plutôt que les posts anciens et rarement vus.  
> - A besoin d'un service ininterrompu même en cas de panne.  
> - A des exigences d'intégrité autour du minimum légal (environ 3 copies)
>
> **Solution Wormhole :**
> - Utilise toutes les ressources qui lui sont offertes, et en permet un ajout facile.  
> - La configuration des systèmes de cache et d'affinités permet de distinguer les serveurs rapides (SSD) et massifs (HDD) et d'utiliser au mieux leur potentiel.  
> - Le cluster maintenant installé sur une telle quantité de serveurs, la redondance et l'équilibrage automatique rendent une interruption de service ou une perte de données virtuellement impossibles.  

<br>

Une fois le système mis en place, tout fonctionne automatiquement, garantissant une utilisation simple et sans accroc.   
La configuration par fichier est réutilisable et partageable. Sa clareté la rend facile à comprendre et maintenir même des années après sa mise en place.  
La plasticité du réseau le rend fiable, adaptable et modifiable sans mesures compliquées.  

### Pour les particuliers
La nature **flexible** de Wormhole lui permet un usage pratique même chez les particuliers.   
Marre de chercher vos documents, photos et projets entre votre NAS, votre ordinateur fixe et votre ordinateur portable ?   
Montez en quelques minutes un réseau Wormhole, et vos différents appareils ne font plus qu'un. Vos données sont disponibles sur tous comme si elles y étaient !   
> [!IMPORTANT] Une fois installé, on oublie très vite la présence de Wormhole.   
> Et pourtant, l'enfer de chercher ses données sur différents appareils, les synchroniser ou les sauvegarder est maintenant de l'histoire ancienne.   
> Wormhole fait tout pour vous 😎   
> On vous a volé votre pc portable ? **Vous n'avez pas perdu vos données.**   
> Votre NAS déraille ? **Vous n'avez pas perdu vos données.**   
> Votre ordinateur fixe brule ?! **Vous n'avez pas perdu vos données !**   
> Vous avez un nouvel appareil ? **Une commande, et tout est géré.**   

___

## Spécification détaillée
Cette partie liste toutes les fonctionalités prévues.

### Interface native

Pour une interaction avec le cluster de manière instinctive, l’accès aux données se fait par l’interface d’un dossier virtuel monté par Wormhole. Cela permet de garder les mêmes moyens d’interaction avec les données que avec tout autre système de fichier. Ces dossiers virtuels sont permis par les technologies natives telles que FUSE (Linux) ou WinFSP (Windows).  

### Intégration Universelle

Une des priorités de Wormhole est de rendre le cluster accessible par le plus d’appareils possible afin que le disque virtuel puisse être compatible avec un maximum de méthodes de travail. 
Nos objectifs prioritaires pour l’EIP sont une intégration sur les plateformes suivantes :
- Linux
- Windows
- Mac

Fuse supportant aussi Android fait d’android une plateforme secondaire intéressante à implémenter.  

Pour simplifier l’accès aux plateformes non supportées nativement, une image Docker sera développée.  
Cette image sera proposée avec une configuration Kubernetes pour faciliter notre entrée dans le monde existant de l’informatique distribuée.  

Chaque implémentation sera parfaitement compatible, peu importe sa plateforme. La même configuration et les mêmes paramètres sont utilisés, renforcant la nature universelle du projet.   

### Configuration

Notre projet veut allier rapidité de mise en place et extensibilité de configuration.  
Pour répondre à ces objectifs, nous optons pour la configuration par fichiers. Cette méthode a déjà fait ses preuves pour des services comme Docker et Kubernetes, en permettant le partage, la réutilisation et le versionning. 
Nous pensons utiliser le format TOML, alliant clarté et modernité, et bien intégré dans l'environnement Rust.  

La configuration se veut la plus complète possible pour moduler tous les aspects du cluster. Elle serait donc à plusieurs niveaux :
- Niveau du cluster pour le comportement général.  
- Niveau Pod avec les informations locales et les affinités propres au pod
- Niveau par fichier pour spécifier des exceptions dans leur comportement.  

Voici une liste d’exemples de champs de configurations qui seraient mis à disposition de l’utilisateur.  
*Cette liste n’est pas exhaustive ou définitive.* Notre objectif est de permettre de configurer tout ce qui peut l’être, ce qui explique que la majorité des champs de configuration spécifiques seront définis au cours du projet.  

**Configuration générale :**
- Nom unique du cluster
- Nombre de redondances par fichier
- Stratégie d’ajout (accepter les nouvelles nodes)
- Taille maximale du stockage proposé
- Administration (qui peut modifier la configuration générale)
- Stratégie de panne
  - Si elle n’entrave pas le fonctionnement ou l’intégrité
  - Si elle entrave l’intégrité (manque de redondances, mais aucun fichier perdu)
  - Si elle entrave le fonctionnement (fichiers manquants)

**Configuration par Pod :**
- Limite d’espace de stockage
- Cache local (propension à garder des copies locales pour accélérer l’usage)
- Affinités (prioriser ou éviter un pod pour une tâche)
  - Stockage des redondances
  - Stockage des nouveaux fichiers
  - Stockage des fichiers les plus demandés
  - Stockage des fichiers les moins demandés
- Stratégie de panne locale (réaction si déconnecté du réseau)

**Configuration par fichier :**
- Conserver (force ce Pod à conserver une version locale)
- Ne pas mettre en cache
- Lecture seule
- Nombre de redondances

### Distribution de données

Avec Wormhole, lors de la lecture d’un fichier qui n’est pas présent localement sur la machine, les données seront téléchargées de la machine hôte à la volée. Cela offre plusieurs possibilitées :
- Agir à distance sur le fichier pendant tout le processus (streaming).  
- Créer une copie locale du fichier pendant son usage, avant d’exporter les mises à jour sur le réseau.  

Agir à distance est plus lent (latence) et utilise de la bande passante, mais possède le bénéfice de ne pas utiliser d’espace disque.  
Utiliser une copie locale utilise de l'espace disque, mais permet une performance accrue.  

L’extensibilité de la configuration permet à l’utilisateur de paramétrer ce comportement (et d’autres comportements similaires).  
Il est aussi important de noter que de manière automatique, Wormhole stockera les fichiers sur les nodes le demandant souvent, optimisant ainsi le système entier.  


### Stratégies de gestion (tolérance de panne, redondance et intégrité, performance…)

La gestion des données est une question complexe, et elle l’est encore plus de grandes infrastructures telles que celles que Wormhole peut opérer. Ce n’est pas pour rien que les entreprises ont des équipes entières consacrées au sujet.  

Les exigences pouvant changer du tout au tout selon le cas d’usage, Wormhole permet de configurer des stratégies à adopter face à différents sujets.  

#### Conflits de données :

La modification simultanée d’un même fichier par plusieurs nodes peut causer des conflits. Il n’existe pas de méthode de résolution de conflits parfaite et universelle. 
L’utilisateur pourra alors choisir parmi une liste de stratégies qui contiendra (sans s’y limiter) :
- Ecraser (garder la version écrite en dernier)
- Garder deux copies

#### Intégrité des données et service ininterrompu (cas général) :

Il est généralement important d’assurer l’intégrité de ses données en cas de panne. Répartir des copies des fichiers sur des machines différentes du réseau permet de garantir leur intégrité en cas de défaillance.  
Non seulement cela, mais cette réplication permet au réseau de continuer son service sans interruption ou disparition de fichiers, même temporaire, lors d'une malfonction.  

Ce procédé porte le nom de redondance et a tout de même le défaut de consommer un espace disque important.  
Selon son usage, l’utilisateur pourra activer ou non ce procédé et choisir le nombre de réplicas par fichier.  
Générer un nombre important de copies peut être une opération lourde pour le cluster. L’utilisateur pourra donc moduler la fréquence de mise à jour des copies.  
Le système placera judicieusement ces copies là où elles sont le plus utilisées pour améliorer les performances.

#### Intégrité et plan de continuité (cas de crise) :

La décentralisation et l’usage de la redondance réduisent grandement la probabilité d’incident majeur.  
Cependant, Wormhole permet de définir les stratégies à adopter en cas de malfonction généralisée.  

Les situations sont divisées en trois catégories : 
- **Situation favorable** :  
Pas de pertes de fichiers, le cluster dispose de l’espace nécessaire pour se rééquilibrer et recréer les redondances manquantes.  
*Abordé dans la section **intégrité des données et service ininterrompu (cas général)***

- **Situation mitigée** :  
  Pas de pertes de fichiers, mais le cluster manque d’espace pour s’équilibrer et recréer la redondance nécessaire.  
  
- **Situation grave** :  
Fichiers manquants sur le réseau, fonctionnement habituel entravé.  

Pour chaque situation, l’utilisateur peut configurer une réaction appropriée.  
Exemples de réactions (non exhaustif) : 
- Ralentir / limiter le trafic
- Geler le réseau (lecture seule) jusqu’à résolution du problème ou action de l’administrateur.
- Baisser le nombre de redondances pour augmenter l’espace libre et poursuivre le service autant que possible.
- Stopper tout.

### Optimisation et répartition des charges

La structure décentralisée en maillage mutualise les capacités et offre de belles perspectives d’optimisation de la performance.  
Le système sera capable de gérer “intelligemment” son infrastructure, par exemple :
- Placer les fichiers et leur redondances sur les nodes les utilisant le plus.
- Transferts parallèles (télécharger différentes parties d’un même fichier depuis deux nodes ou plus, doublant la vitesse de transfert. Il en va de même pour l’upload).  
- Répartition des opérations lourdes.  
*Exemple* : si le nombre de redondances est élevé, chaque node fera le transfert à seulement deux autres, qui feront de même, etc, évitant ainsi à une seule node de faire tous les transferts.  

L’utilisateur pourra aussi moduler ses besoins pour soulager le réseau.  
*Exemple* : Réduire la fréquence de réplication des fichiers, pour éviter de propager une opération lourde sur le cluster à chaque édition.  

### Flexibilité et fonctions additionnelles
Le cluster peut être modifié sans être interrompu. Cela facilite les évolutions et permet :
- L’ajout de nouvelles nodes
- Le retrait de nodes
- La modification de la configuration

Le cluster s'équilibre automatiquement selon le nouveau contexte, sans perturber les services pouvant dépendre des données.  

Il est aussi possible de créer des Pods dit “Clients”. Ceux-ci peuvent accéder aux fichiers du cluster sans pour autant devenir une maille du réseau.  
Ils peuvent alors se connecter ou déconnecter à la volée sans perturber le système, ce qui les rend adaptés à un déploiement à grande échelle. *(Par exemple, les ordinateurs portables des collaborateurs de l’entreprise.)*


## Spécification Non-Technique

Sujets de dévelopment:

***Obligatoire :***

### Évaluer et Intégrer les nouvelles technologies
Nous utiliserons une stack technique **récente**, avec une communauté active et **axée performance / sécurité**.  
> [!TIP] Rust nous parrait le choix de langage le plus approprié.  
> Nous resterons à l'écoute des évolutions de ce langage comme des autres pour ajuster nos choix.  

**Nous suivrons l'apparition de nouvelles plateformes** et de leur pertinence pour une potentielle intégration native.  
> [!NOTE] Les plateformes prioritaires sont actuellement :
> - Linux
> - Windows
> - Mac


**Nous explorerons les différents protocoles réseau qui pourraient nous servir au mieux**, tant pour leur vitesse que pour leur fiabilité.   
Cela va de soit aussi pour les protocoles d'accès.  
> [!IMPORTANT] Dans le cadre de l'EIP, nous utiliserons les protocoles natifs pour les systèmes précédement cités.  
> Mais nous sommes ouverts à l'intégration future de protocoles ouverts pour étendre nos compatibilités.  


Nous tiendrons un environnement de développement à jour pour accélérer les temps d’itérations.  

<br>

___

***Optionnels :***
### Protéger et améliorer notre technologie
**Nous sélectionnerons et appliquerons une licence de développement open source qui servira au mieux notre projet.**   
> [!NOTE] Nous réfléchissons à des licences ouvertes pour les particuliers et payantes pour l'usage commercial / d'entreprises.   
> Ces licences ont le bénéfice de ne pas entraver la démocratisation du projet tout en ouvrant la possibilité de le rentabiliser.  


Nous validerons soigneusement nos dépendances afin de :
- Respecter leurs licences
- Respecter nos objectifs de légereté, performance et multiplateforme
- Limiter notre surface d'attaque
___
### Entretenir les contributions par la communauté
**Nous voulons obtenir rapidement le soutien de la communauté.**  
Cela passe par plusieurs mesures :

#### - Utilisation plaisante et accessible
Bien que notre outil reste technique et qu'il ne disposera pas de notion "UI/UX" à proprement parler, nous ferons de notre mieux pour le rendre intuitif dès la première utilisation, et surtout pour les besoins courrants et simples. Tout cela afin de ne pas décourager les personnes pouvant s'intérésser au projet.  
> [!TIP] L'environnement de développement Rust est une bonne inspiration.  
> Rust est intrasequement une notion technique, mais la "developer experience" est une préoccupation.  
> - La documentation est claire.  
> - Le compilateur détaille et explique les erreurs ou warnings de manière claire.  
> - Vient avec une suite d'outils (formatting, cross-compilation, intégration Visual Studio Code...)
>
>
> Tout ceci joue probablement une grande part dans la popularité de Rust, et est inspirant pour un projet comme le notre.  

> [!CAUTION] Notre EIP reste un EIP technique.  
> L'expérience utilisateur fera partie de nos préoccupations car elle constitue une bonne stratégie, mais elle reste au second plan face aux objectifs techniques précisés dans notre spécification.   
> Les objectifs "qualité" (documentation claire, CLI bien pensée) seront bien sur intégrés, mais les objectifs additionnels (ex. suite d'outils) ne feront pas partie des sprints ou objectifs de l'EIP.  

#### - Clareté technique
Nous ciblerons un public qui souhaite des améliorations au projet et possède la volonté de les faire.   
Cela implique :
- Maintenir une documentation technique publique et claire, pour faciliter le développement par des tiers.  
- Notre projet sera publique sur GitHub et incitera à la contribution.  
- La RoadMap sera publiée pour donner à chaque contributeur potentiel une idée de l'avancement et de l'activité du projet.  
- Dans la mesure du possible, nous parlerons de notre projet sur des groupes internet centrés autour du sujet (Reddit, Discord...)