# Wormhole

Wormhole is a data decentralisation solution. It aims to create one single virtual storage space between many computers.

You can think if it as the *Kubernetes* of storage space.

---

## Overview

Wormhole is an open-source project designed to provide a decentralized, scalable, and user-friendly data storage solution. By creating a virtual file system that spans multiple machines, Wormhole enables seamless data sharing and redundancy without the need for complex infrastructure management. Whether you're a small startup, a large enterprise, or an individual managing personal devices, Wormhole simplifies data storage and access with a native, intuitive interface.

This `README` provides an introduction to Wormhole, setup instructions, and links to detailed documentation. For a comprehensive understanding of the project's goals and technical details, refer to the [Technical Specification](docs/technical/technical_spec.md).

---

## Our Idea

Inspired by great declarative softwares of modern times like Docker, we are aiming to provide users with a very flexible solution, allowing all kinds of usages while staying declarative, simple and shareable.

## Concept

We want Wormhole to be as seamless as possible for final users. The storage space takes the shape of a simple folder. No need to create or mount any partition, the virtual space is mounted in place, where you want in your file tree.

For users and other softwares, the files behave like any normal files, while they are in fact shared and moved accross all nodes (differents computers) of the network.

---

## Features

- **Decentralized Storage**: Combine multiple machines into a single virtual storage space.
- **Native Integration**: Files appear as local files, requiring no changes to existing applications.
- **Scalability**: Suitable from small local networks to large enterprise infrastructures.
- **Redundancy**: Configurable data replication to ensure integrity and availability.
- **Flexibility**: Supports dynamic addition/removal of nodes without service interruption.
- **Configuration**: Declarative, file-based configuration using TOML for ease of use and sharing.

For detailed use cases and technical details, see the [Technical Specification](docs/technical/technical_spec.md).

---

## Getting Started

Follow these steps to set up a Wormhole network on your machine. For detailed instructions, refer to the [Installation Guide](docs/getting-started/installation.md).

Wormhole uses two binaries:
 - "wormholed" the background service managing the pods
 - "womrhole" the command line interface, acting as an interface with the service

 Wormhole being still in heavy developpement, the project still require to build the project from source.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) installed.
- Clone the source code.
- Optional: [Docker](https://docs.docker.com/get-docker/) for containerized deployment.

### How to build

Run cargo build command:
```
cargo build --release
```

Move the binaries where needed, they can be found under `target/release/wormhole` and `target/release/wormholed`

### How to run

Launch a new service, the service is started automatically
```
./wormholed 127.0.0.1:8081
^---------- ^-------------
 Service     Optional address with a default at 127.0.0.1:8081
```

Create a new Wormhole network
The new pod being created with any other connection it will automaticaly create a new network
```
./wormhole 127.0.0.1:8081           new      pod_name  --path dir1/ -i 127.0.0.10:8081
^--------- ^-------------           ^--      ^-----    ^------- ^-----------------
 CLI        Optional service address Command  Pod Name  Directory Pod Address
```

Join an existing Wormhole network
```
./wormhole 127.0.0.2:8081 new pod_name2  --path dir2/ -i 127.0.0.11:8081 -u 127.0.0.10:8081
                                                                         ^-----------------
                                                                          Existing pod address
```

For a step-by-step guide to setting up a multi-pod network, see the [CLI Usage Guide](docs/getting-started/memo-cli.md).
For a more complex Docker-based deployment, refer to the [Docker Guide](docs/getting-started/docker-guide.md).

---

### CLI Commands Overview
```
  start        Start the service
  stop         Stop the service
  template     Create a new network (template)
  new          Create a new pod and join a network if he have peers in arguments or create a new network
  inspect      Inspect a pod with its configuration, connections, etc
  get-hosts    Get hosts for a specific file
  tree         Tree the folder structure from the given path and show hosts for each file
  remove       Remove a pod from its network
  apply        Apply a new configuration to a pod
  restore      Restore many or a specifique file configuration
  interrupt    Stops the service
  help         Print this message or the help of the given subcommand(s)
```

## Documentation

The Wormhole documentation is organized into the following sections:

- **Getting Started**:
  - [Installation Guide](docs/getting-started/installation.md): Step-by-step instructions for setting up Wormhole.
  - [CLI Usage Guide](docs/getting-started/memo-cli.md): How to use the Wormhole CLI to create and manage networks.
  - [Docker Guide](docs/getting-started/docker-guide.md): Instructions for running Wormhole in Docker containers.
- **User Guide**:
  - [Configuration Guide](docs/user-guide/configuration.md): How to configure Wormhole using TOML files.
  - [Glossary](docs/user-guide/glossary.md): Definitions of key terms and concepts.
- **Technical Documentation**:
  - [Technical Specification](docs/technical/technical_spec.md): Detailed explanation of Wormhole’s architecture and features.
  - [Technical Specification (French)](docs/technical/technical_spec_fr.md): French version of the technical specification.
  - [Configuration Details](docs/technical/configuration/): In-depth configuration options (main, pod, and file-level).
- **Beta Testing**:
  - [Beta Test Plan](docs/beta-testing/beta-test-plan.md): Scenarios and criteria for testing the beta version.
- **UML Diagrams**:
  - Located in [docs/uml/](docs/uml/): Visual representations of Wormhole’s architecture.

---

## Contributing

Wormhole is an open-source project, and we welcome contributions from the community! To get involved:

1. Read the [Technical Specification](docs/technical/technical-spec.md) to understand the project’s goals and architecture.
2. Check the [Beta Test Plan](docs/beta-testing/beta-test-plan.md) to see testing scenarios and provide feedback.
3. Report issues or suggest improvements via the [GitHub Issues](https://github.com/<your-repo>/issues) page.
4. Submit pull requests with code contributions, following the guidelines in [Code Architecture](docs/technical/architecture/code-architecture.md).

For terminology, refer to the [Glossary](docs/user-guide/glossary.md) to understand key concepts like nodes, pods, and networks.

---

## Known Issues and Limitations

The current beta version has some known limitations, detailed in the [Beta Test Plan](docs/beta-testing/beta-test-plan.md). Key issues include:

- **Windows Support**: Incomplete, with some features not fully implemented.
- **Documentation**: Some sections are incomplete and being expanded.

We are actively working on these issues and encourage community feedback to improve Wormhole.

---

## License

Wormhole is licensed under the [The GNU Affero General Public License](LICENSE.txt). See the license file for details.

---

## Acknowledgments

Wormhole is developed by Axel Denis, Julian Scott, Ludovic de Chavagnac, and Arthur Aillet. We thank all contributors and testers for their support in making Wormhole.
