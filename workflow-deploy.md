# Workflow: Jero-Mu Talos Cluster Initialization (Deploy)

This workflow defines the steps executed by `unyeong` to bootstrap a two-node hybrid (Control Plane + Worker) Talos cluster.

## Objective
Configure and initialize a stable Kubernetes environment on node-1 (Xeon) and node-2 (N100).

## Node Inventory

### Node 1 (node-1.talos.local)
- **1G Link:** `<MGMT_IP_NODE1>` (Management)
- **10G Link:** `<MGMT_IP_NODE1>1` (Worker/Data)
- **OS Disk:** Smallest available drive.
- **Data Disk:** Biggest drives (Configured as ZFS Mirror/RAID1).

### Node 2 (node-2.talos.local)
- **1G Links:** `<MGMT_IP_NODE2>`, `<MGMT_IP_NODE2_2>` (Management)
- **10G Link:** `<MGMT_IP_NODE1>2` (Worker/Data)
- **OS Disk:** SSD.
- **Data Disk:** USB Drive.

## Steps

### 1. Configuration Generation
Generate the core cluster secrets and machine configuration patches.
- **Secrets:** `talosctl gen secrets -o ./talos/secrets.yaml`
- **MachineConfigs:** Generate base configs using `talosctl gen config jero-mu https://kube.jero-mu.talos.local:6443`.

### 2. Node Configuration & Application
Apply hardware-specific patches (network interfaces, disk layouts) to each node.
- **Action:** Apply `./talos/node-1.yaml` to `<MGMT_IP_NODE1>`.
- **Action:** Apply `./talos/node-2.yaml` to `<MGMT_IP_NODE2>`.

### 3. Cluster Bootstrap
Kickstart the Kubernetes control plane on the first node.
- **Command:** `talosctl bootstrap --nodes <MGMT_IP_NODE1> --endpoints <MGMT_IP_NODE1> --talosconfig ./talos/talosconfig`

### 4. Client Persistence
Ensure management tools are configured to point to the new cluster.
- **talosctl:** Merge `./talos/talosconfig` into standard search path or keep locally.
- **kubectl:** Download admin kubeconfig: `talosctl kubeconfig -n <MGMT_IP_NODE1> --talosconfig ./talos/talosconfig .`

## Verification
- `kubectl get nodes` shows both nodes in `Ready` status with roles `control-plane,worker`.
- Verify ZFS mirror health on node-1.
- Verify USB storage accessibility on node-2.
