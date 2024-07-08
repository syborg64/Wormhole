# Configuration d'un Pod

Cette configuration est spécifique au pod.<br>
Le cluster s'y conformera dans la mesure du possible, mais pourra l'ignorer si nécessaire (voir [stratégies d'urgence](../strategies/emergency.md)).<br>
Cette configuration est principalement utilisée pour ajuster la stratégie du cluster à un niveau local.

> [!AVERTISSEMENT]
> La configuration d'un cluster hétérogène est considérée comme une configuration avancée. Elle peut entraîner des goulets d'étranglement, une baisse des performances ou une gestion non sécurisée des données. Nous vous recommandons de laisser le système se gérer lui-même si vous n'avez pas de connaissances ou de besoins spécifiques.

## Stockage
>
> [!NOTE] [stackage]

*default: max-disk-space*<br>
Le cluster tiendra compte de ces informations lorsqu'il choisira un pod pour stocker les données.
> [!CONSEIL]
>
> - Peut être utilisé si le cluster compense une défaillance (par exemple, en stockant des redondances). [Plus](../stratégies/emergency.md)
> - Peut être temporairement épuisé si un utilisateur local tire beaucoup de données dans le point de montage.
> - Peut être temporairement utilisé en cas de mouvements de données importants.
> - Ne sera pas utilisé si l'utilisateur pousse trop de données pour le cluster. L'utilisateur sera alerté lorsqu'il atteindra la capacité maximale.
> - Ne sera pas utilisé si l'administrateur crée une stratégie qui génère trop de données (par exemple, en augmentant la redondance). L'administrateur sera alerté en cas de règle irréalisable.

---

**max-disk-space**: Mo<br>
*default: 95% de l'espace disque disponible au point de montage du pod.*<br>
Cette taille **ne peut pas** être dépassée par les actions de Wormhole<br>
> [!NOTE]
> peut être utilisé temporairement lorsque l'utilisateur charge un nouveau fichier local dans le système, le temps qu'il soit déchargé dans le cluster.

> [!IMPORTANT]
> Si un fichier demandé est trop volumineux pour être extrait lors de la demande, le pod devra décharger les données locales vers le cluster, ce qui augmentera le temps de réponse. Si le cluster n'est pas en mesure d'effectuer ce transfert de données, l'utilisateur ne pourra pas accéder à ce fichier.

## Sratégie
>
> [!NOTE] [stratégie]

**redundancy-priority**: nombre<br>
*default: 0*<br>
Lors du choix d'un pod pour stocker la redondance, les pods les plus prioritaires seront utilisés en premier.

---

**cache**: nombre<br>
*default: 2*<br>

0. décharger tout
1. présélection légère gérée par le système
2. préréglage lourd géré par le système
3. télécharger tout

> [!NOTE]
> Ce paramètre est plus utile pour le nœud utilisant le système de fichiers que pour le cluster.<br>
> Néanmoins, le fait d'avoir plus de mémoire cache peut donner au système plus de liberté lors de l'extraction des données et contribuer à améliorer les performances des clusters.

<br>
<br>
<br>
<br>

___
___
___

> [!NOTE]
> Met en évidence les informations que les utilisateurs doivent prendre en compte, même s'ils ne font qu'effleurer le sujet.

> [!CONSEIL]
> Informations facultatives pour aider l'utilisateur à mieux réussir.

> [!IMPORTANT] > Informations cruciales nécessaires à la réussite de l'utilisateur.

> [!AVERTISSEMENT]
> Contenu critique exigeant une attention immédiate de la part de l'utilisateur en raison des risques potentiels.

> [!ATTENTION] > Conséquences négatives potentielles d'une action.
