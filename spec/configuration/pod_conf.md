# Pod configuration
This configuration is pod-specific.<br>
The cluster will comply with it while possible, but can override if necessary (see [emergency strategies](../strategies/emergency.md))<br>
This configuration is mostly used to adjust the cluster strategy at a local level.

> [!WARNING]
> Heterogenous cluster configuration is considered an advanced configuration. This can lead to bottlenecks, decreased performance or unsecure data management. We recommend to let the system manage itself if you have no specific knowledge or needs.

## Storage
> [!NOTE] [storage]



*default: max-disk-space*<br>
The cluster will take this information into account when choosing a pod to store data.
> [!TIP]
> - Can be exeeded if the cluster is compensating for failure (E.g. storing redundancies). [More](../strategies/emergency.md)
> - Can be temporarily exeeded if a local user is pulling a lot of data in the mountpoint.
> - Can be temporarily exeeded when encountering large data movements.
> - Won't be exeeded if user is pushing too much data for the cluster. The user will be alerted for hitting max capacity.
> - Won't be exeeded if admin create a strategy that generates too much data (E.g. increase redundancy). The admin will be alerted for unfeasible rule.


---

**max-disk-space**: Mo<br>
*default: 95% of node available disk space at pod mountpoint.*<br>
This size **cannot** be exeeded by Wormhole actions.<br>
> [!NOTE]
> Can be temporarily exeeded when the user load a new local file in the system, time for it to be unloaded to the cluster.

> [!IMPORTANT]
> If an asked file is too large to be pulled when asked, the pod will have to unload local data to the cluster, leading to increased response time. If the cluster for this data transfer, the user will be unable to access this file.

## Srategy
> [!NOTE] [strategy]

**redundancy-priority**: number<br>
*default: 0*<br>
When choosing a pod to store redundancy, pods higher priority will be used first.

---

**cache**: number<br>
*default: 2*<br>

0. unload all
1. system managed light preset
2. system managed heavy preset
3. download all

> [!NOTE]
> This parameter is more useful for the node using the filesystem than for the cluster.<br>
> Even so, having more cache can give the system more freedom when pulling data and help achieving greater cluster performance.

<br>
<br>
<br>
<br>

___
___
___

> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!TIP]
> Optional information to help a user be more successful.

> [!IMPORTANT]
> Crucial information necessary for users to succeed.

> [!WARNING]
> Critical content demanding immediate user attention due to potential risks.

> [!CAUTION]
> Negative potential consequences of an action.
