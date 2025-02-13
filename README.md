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
```
cargo run --bin service -- ./virtual       "127.0.0.1:8080" "127.0.0.2:8080"
^---------------------     ^-------        ^--------------  ^---------------
Build and run              where to mount  host ip          other ips (multiple possible)
```
