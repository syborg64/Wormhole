# Wormhole
Wormhole is a data decentralisation solution. It aims to create one single virtual stockage space between many computers.

You can think if it as the *Kubernetes* of storage space.

## Our idea
Inspired by great declarative software of modern times like docker, we are aming to provide users with a very flexible solution, allowing all kinds of usages while staying declarative, simple and shareable.

## The concept
We want Wormhole to be as transparent as possible for final users. The storage space take shape of a simple folder. No need to create or mount any partition, the virtual space is mounted in place, where you want in your file tree.

For users and other softwares, the files behaves like any normal files, while they are in fact shared and moved accross all nodes (differents computers) of the network.

## How to launch
(this is for demo purposes and will be changed)

The connection between the cli and the service is made by default on the address 127.0.0.1:8081 but can be modified by adding the new address in the command just after the binary (to be done for both)

```
cargo run --bin wormholed
^---------------------
Build and run
```

Create a new Wormhole network
```
cargo run --bin wormhole new pod_name -C virutal1/ -i 127.17.0.1:8081
^---------------------       ^--                ^-----       ^---------------
Build and run                 command            directory     host ip
```

Join an existing Wormhole network
```
cargo run --bin wormhole new pod_name2 -C virutal2/ -i 127.17.0.2:8081 -u 127.17.0.1:8081 -a 127.17.0.3:8081 127.17.0.4:8081 127.17.0.5:8081
^---------------------       ^--                ^-----       ^----------       ^-------------     ^-----------------------------------------------
Build and run                 command            directory     host ip         ip of node to join     additionnal host
```
