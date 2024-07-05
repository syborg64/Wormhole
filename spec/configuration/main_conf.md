# Main Wormhole configuration
Main configuration for a Wormhole network.
This configuration defines the general behavior of the network and all its related information.
> [!IMPORTANT]
> The system will always try to comply with rules defined in here. In case of conflict, they have absolute priority over almost all [individual pod configuration](./pod_conf.md) rules.
> See [emergency strategies](../strategies/emergency.md) for details.

## General
> [!NOTE] [wormhole]

> [!CAUTION] Mandatory
> **name**<br>
> Short, simple name of the network

---

**access**: open | demand | whitelist | blacklist<br>
*default:demand*<br>
Defines how new pods should join the network.

---

## Network
> [!NOTE] [network]

**frequency**: seconds<br>
*default:0(smart)*<br>
Time during which outgoing write requests are locally stored before being sent all at once.
Prevents network floods when creating lot of files fastly.
> [!NOTE]
> A value of 0 let the system manage itself, balancing over a base frequency of 1sec depending on current use.


---

## Redundancy
> [!NOTE] [redundancy]

> [!TIP]
> Redundancy is a very important parameter when aiming to securing data. This value defines the number of replications of a file over many nodes. Having at least one replica enables :
> - Failure-safe data storage.<br>
> Upon node failure, no data is lost, and the cluster will rebalance itself.
> Always-on system.<br>
> - Even while rebalancing after failure, data is still available and users keeps a seamless experience.

**amount**: number<br>
*default:0*<br>
Number of replicas for one file. Replicas are made for safety and thus stored on different nodes.
> [!WARNING]
> - Can't exeed the number of actives nodes.
> - Storage needs increase linearly.

> [!TIP]
> The system will smartly store replicas on nodes where the file is regularly requested to speed up the system :D

**strategy**: number<br>
*default:2*<br>
Instantly replicate every change can cause a lot of useless stress on the cluster. You can set a strategy depending on your needs.

0. Instantly replicate all operations<br>
If you can't afford to lose even one minute of data upon failure
1. System managed
Will target inactivity periods for a file, preventing the propagation of too many minor writes when using a file.<br>
Uses min-replication-time & max-replication-time
2. Fixed
Replicates a file every max-replication-time (if the file got a modification since last time)

---

**min-replication-time**: minutes<br>
*default:10*<br>
> [!NOTE] Used by the redundancy strategy when system managed

Minimum time before repropagating a save when system managed.

---

**max-replication-time**: minutes<br>
*default:120*<br>
> [!NOTE] Used by the redundancy strategy when system managed.<br>Used by the redundancy strategy when fixed.

Maximum time before repropagating a save when system managed.