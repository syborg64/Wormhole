# Wormhole

Wormhole is a data decentralisation solution. It aims to create one single virtual stockage space between many computers.

You can think if it as the *Kubernetes* of storage space.

---

## Overview

Wormhole is an open-source project designed to provide a decentralized, scalable, and user-friendly data storage solution. By creating a virtual file system that spans multiple machines, Wormhole enables seamless data sharing and redundancy without the need for complex infrastructure management. Whether you're a small startup, a large enterprise, or an individual managing personal devices, Wormhole simplifies data storage and access with a native, intuitive interface.

This `README` provides an introduction to Wormhole, setup instructions, and links to detailed documentation. For a comprehensive understanding of the project's goals and technical details, refer to the [Technical Specification](docs/technical/technical-spec.md).

---

## Our Idea

Inspired by great declarative software of modern times like docker, we are aming to provide users with a very flexible solution, allowing all kinds of usages while staying declarative, simple and shareable.

## The Concept

We want Wormhole to be as transparent as possible for final users. The storage space take shape of a simple folder. No need to create or mount any partition, the virtual space is mounted in place, where you want in your file tree.

For users and other softwares, the files behaves like any normal files, while they are in fact shared and moved accross all nodes (differents computers) of the network.

---

## Features

- **Decentralized Storage**: Combine multiple machines into a single virtual storage space.
- **Native Integration**: Files appear as local files, requiring no changes to existing applications.
- **Scalability**: Suitable for small local networks to large enterprise infrastructures.
- **Redundancy**: Configurable data replication to ensure integrity and availability.
- **Flexibility**: Supports dynamic addition/removal of nodes without service interruption.
- **Configuration**: Declarative, file-based configuration using TOML for ease of use and sharing.

For detailed use cases and technical details, see the [Technical Specification](docs/technical/technical-spec.md).

---

## Getting Started

Follow these steps to set up a Wormhole network on your machine. For detailed instructions, refer to the [Installation Guide](docs/getting-started/installation.md).

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) and [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.
- Optional: [Docker](https://docs.docker.com/get-docker/) for containerized deployment.

### How to Launch

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

For a step-by-step guide to setting up a multi-pod network, see the [CLI Usage Guide](docs/getting-started/memo-cli.md). For Docker-based deployment, refer to the [Docker Guide](docs/getting-started/docker-guide.md).

---

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
  - [Technical Specification](docs/technical/technical-spec.md): Detailed explanation of Wormhole’s architecture and features.
  - [Technical Specification (French)](docs/technical/technical-spec-fr.md): French version of the technical specification.
  - [Code Architecture](docs/technical/architecture/code-architecture.md): Overview of the codebase structure.
  - [Logical Architecture](docs/technical/architecture/logical-architecture.md): Logical design of Wormhole’s system.
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

## Community and Support

Join our community to stay updated and get support:

- **GitHub Repository**: [github.com/Agartha-Software/Wormhole](https://github.com/Agartha-Software/Wormhole)
- **Discussions**: Participate in [GitHub Discussions](https://github.com/Agartha-Software/Wormhole/discussions/landing) for questions and ideas.
- **Issue Tracker**: Report bugs or feature requests at [GitHub Issues](https://github.com/Agartha-Software/Wormhole/issues).

---

## License

Wormhole is licensed under the [The GNU Affero General Public License](LICENSE.txt). See the license file for details.

---

## Acknowledgments

Wormhole is developed by Axel Denis, Julian Scott, Ludovic de Chavagnac, and Arthur Aillet. We thank all contributors and testers for their support in making Wormhole a robust and user-friendly solution.