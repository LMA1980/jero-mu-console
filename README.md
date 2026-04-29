# Unyeong

Unyeong (운영/運) is a cluster management utility for Talos Linux metal clusters. It provides automated, deterministic configuration patching ("Match & Patch") to ensure reproducible deployments in heterogeneous hardware environments.

## Features
- **Deterministic Identity**: Uses MAC-address-based profiling to map hardware to node roles.
- **Software-Defined Hardware**: Injectable patches for NIC, storage, and host resolution configuration.
- **Cluster Self-Resolution**: Automatically generates internal DNS entries for cluster inter-communication.
- **Readiness Verification**: Built-in health checks for Talos and Kubernetes control planes.

## License & Contribution
- This project is licensed under the **Business Source License 1.1** (transitioning to AGPLv3 on April 28, 2029).
- Please refer to `CLA.md` for contributor license agreement terms.
