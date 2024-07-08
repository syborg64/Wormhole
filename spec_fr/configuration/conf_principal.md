# Configuartion principal de Wormhole

Configuration principal du réseau de Wormhole.
Cette configuration definie le comportement général du réseau et toutes les informations qui s'y rapportent.

> [!IMPORTANT]
> Le système essaiera toujours de se conformer aux règles définies ici. En cas de conflit, elles ont la priorité absolue sur presque toutes les règles de [configuration individuelle du pod](./pod_conf.md).
> Voir [stratégies d'urgence](../strategies/emergency.md) pour plus de détails.

## Generale
>
> [!NOTE] [wormhole]

> [!ATTENTION] Obligatoire
> **name**<br>
> Nom court et simple du réseau

---

**access**: ouvert | demande | whitelist | blacklist<br>
*default: demande*<br>
Définie comment un nouveau Pod peux rejoindre le réseau

---

## Réseau
>
> [!NOTE] [réseau]

**frequency**: secondes<br>
*default: 0(intelligent)*<br>
Durée pendant laquelle les demandes d'écriture sortantes sont stockées localement avant d'être envoyées en une seule fois.
Prévient les inondations du réseau lors de la création rapide d'un grand nombre de fichiers.
> [!NOTE]
> Une valeur de 0 permet au système de se gérer lui-même, en s'équilibrant sur une fréquence de base de 1 seconde en fonction de l'utilisation actuelle.
---

## Redondance
>
> [!NOTE] [redondance]

> [!CONSEIL]
> La redondance est un paramètre très important lorsqu'il s'agit de sécuriser des données. Cette valeur définit le nombre de réplications d'un fichier sur plusieurs nœuds. Le fait d'avoir au moins une réplique permet :
>
> - Stockage des données à l'abri des défaillances.<br>
> En cas de défaillance d'un nœud, aucune donnée n'est perdue et la grappe se rééquilibre d'elle-même.
> Système toujours actif.<br>
> - Même en cas de rééquilibrage après une panne, les données sont toujours disponibles et les utilisateurs bénéficient d'une expérience transparente

**amount**: nombre<br>
*default:0*<br>
Nombre de replicas pour un fichier. Replicas sont effectuées pour des raisons de sécurité et sont donc stockées sur des nœuds différents.
> [!AVERTISSEMENT]
>
> - On ne peut pas dépasser le nombre de nœuds actifs.
> Les besoins en stockage augmentent de façon linéaire.

> [!CONSEIL]
> Le système stockera intelligemment des répliques sur les nœuds où le fichier est régulièrement demandé afin d'accélérer le système :D

**strategy**: nombre<br>
*default:2*<br>
Répliquer instantanément chaque changement peut causer beaucoup de stress inutile sur le cluster. Vous pouvez définir une stratégie en fonction de vos besoins.

1. Répliquer instantanément toutes les opérations<br>
Si vous ne pouvez pas vous permettre de perdre ne serait-ce qu'une minute de données en cas de défaillance
2. Géré par le système
Cible les périodes d'inactivité d'un fichier, empêchant la propagation d'un trop grand nombre d'écritures mineures lors de l'utilisation d'un fichier.<br>
Utilise le temps de réplication minimum et le temps de réplication maximum
3. Fixe
Réplique un fichier tous les max-replication-time (si le fichier a été modifié depuis la dernière fois).

---

**min-replication-time**: minutes<br>
*default:10*<br>
> [!NOTE] Utilisé par la stratégie de redondance lorsque le système est géré.

Temps minimum avant de repropager une sauvegarde lorsque le système est géré.

---

**max-replication-time**: minutes<br>
*default:120*<br>
> [!NOTE] Utilisé par la stratégie de redondance lorsque le système est géré.<br>Utilisé par la stratégie de redondance lorsque le système est fixe.

Délai maximum avant de repropager une sauvegarde lorsque le système est géré.
