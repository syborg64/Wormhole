# Wormhole CLI Usage Guide to setup a testing environement

This guide explains how to set up a new Wormhole network with multiple pods using the command line interface (CLI). The steps are designed to be simple and clear, requiring no consultation of external resources beyond this document.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) installed.
- Optional: [Docker](https://docs.docker.com/get-docker/) for containerized deployment.

## Step 1: Create virtual folders

Create three virtual folders to simulate different pods on your machine:

```
mkdir virtual1 virtual2 virtual3
```

## Step 2: Generate configuration templates

For each virtual folder, generate a configuration template using the CLI:

```
RUST_LOG=wormhole=debug cargo run --bin wormhole -- template -C virtual1
RUST_LOG=wormhole=debug cargo run --bin wormhole -- template -C virtual2
RUST_LOG=wormhole=debug cargo run --bin wormhole -- template -C virtual3
```

These commands create configuration files in each folder.

## Step 3: Start Wormhole services

Open three different terminals and run the following command in each to start three Wormhole services. These services will listen on 127.0.0.1:8081, 127.0.0.1:8082, and 127.0.0.1:8083 respectively, as configured in their respective virtual folders.

```
RUST_LOG=wormhole=debug cargo run --bin wormholed
```

**Note**: You may need to run this command from within each virtual folder (e.g., `cd virtual1; RUST_LOG=wormhole=debug cargo run --bin wormholed`) or specify the configuration directory with `-C virtual1`, `-C virtual2`, etc., depending on how `wormholed` is implemented. This guide uses the exact command you provided, assuming the configuration is handled elsewhere or that the services automatically bind to the specified ports.

## Step 4: Create a new network

In a new terminal, create a network with the first pod using the following command:

```
RUST_LOG=wormhole=debug cargo run --bin wormhole -- 127.0.0.1:8081 new default -C virtual1 -i 127.0.0.10:8080
```

This command initializes a network named "default" with the first pod.

## Step 5: Join the network with other pods

Add the second and third pods to the network using the following commands:

For the second pod:
```
RUST_LOG=wormhole=debug cargo run --bin wormhole -- 127.0.0.1:8082 new default -C virtual2 -i 127.0.0.11:8080 -u 127.0.0.10:8080
```

For the third pod:
```
RUST_LOG=wormhole=debug cargo run --bin wormhole -- 127.0.0.1:8083 new default -C virtual3 -i 127.0.0.12:8080 -u 127.0.0.10:8080
```

These commands connect the pods to the network using the address of the first pod.

## Step 6: Verify network connectivity

To test that all pods are properly connected, add a blank file to one pod and check if it appears in the others.

For example, create a file in the first pod's folder:
```
touch virtual1/testfile.txt
```

Wait a few seconds for synchronization, then check the other folders:
```
ls virtual2
ls virtual3
```

You should see `testfile.txt` in both `virtual2` and `virtual3`. If the file appears in all folders, the network is functioning correctly.

## Note for advanced users

To create a third instance on another machine in the same local area network, follow similar steps, adjusting the IP addresses accordingly. For example:

1. On the other machine, create a virtual folder: `mkdir virtual3`
2. Generate the configuration template: `RUST_LOG=wormhole=debug cargo run --bin wormhole -- template -C virtual3`
3. Start the service: `RUST_LOG=wormhole=debug cargo run --bin wormholed` (ensure it listens on the correct IP and port, e.g., 192.168.1.101:8083)
4. Join the network: `RUST_LOG=wormhole=debug cargo run --bin wormhole -- <service_address> new default -C virtual3 -i <pod_address> -u <first_pod_address>`, where `<service_address>` is the address the service is listening on, `<pod_address>` is the address for this pod, and `<first_pod_address>` is the address of the first pod.

For instance, if the first pod is on 192.168.1.100:8080 and the third pod is on 192.168.1.101:8083, you would use:
```
RUST_LOG=wormhole=debug cargo run --bin wormhole -- 192.168.1.101:8083 new default -C virtual3 -i 192.168.1.101:8083 -u 192.168.1.100:8080
```

**Note**: The original commands use loopback aliases (127.0.0.10, etc.), which work for pods on the same machine if configured appropriately. For a different machine, use its actual IP address.

## Conclusion

By following these steps, you have set up a functional Wormhole network with multiple pods and verified their connectivity. This process demonstrates a simple and clear onboarding for new users, without the need for external resources.
