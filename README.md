# WORMHOLE

## Definitions of technical terms

## Configuration

The language chosen for configuration is the TOML, offering excellent clarity.

### [Network Configuration](./spec/configuration/conf_principal.md)

General network configuration
Cloned by newcomers during a join, it defines the network's main rules.
It is absolute, as pod-specific configuration can modulate but not invalidate its action.

### [Configuration per Pod](./spec/configuration/pod_conf.md)

The configuration per pod is effective only for that pod.
It is nevertheless public, to help the network manage all pods.
These rules are only applied if their existence does not invalidate the network configuration. ([see](./spec/details/todo.md)) // TODO

## Architecture

### [Logical architecture](./spec/Architecture/logical_architecture.md)

// TODO

### [Code architecture](./spec/Architecture/code_architecture.md)

// TODO
