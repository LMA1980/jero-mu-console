# Workflow: Jero-Mu Talos Hard Reset (Clean)

This workflow defines the steps executed by `unyeong --clean` to decommission the existing Talos environment and prepare for a fresh installation.

## Objective
Nuke the current Talos cluster, wipe node-level state, and clear all local deployment artifacts.

## Steps

### 1. Remote Node Reset
Trigger a factory reset on all identified node interfaces to purge persistent configuration and data.
- **Targets:** 
    - Node 1: `<MGMT_IP_NODE1>`, `<MGMT_IP_NODE1>1`
    - Node 2: `<MGMT_IP_NODE2>`, `<MGMT_IP_NODE2_2>`, `<MGMT_IP_NODE1>2`
- **Command:** `talosctl reset --nodes <IPs> --reboot --insecure`

### 2. Local Artifact Cleanup
Delete local configuration and state files to prevent stale settings from interfering with the new deployment.
- **Action:** Remove all files in `./talos/` (except templates).
- **Action:** Delete `./.env` file.
- **Action:** Remove any local `kubeconfig` or `talosconfig` context references.

## Verification
- Verify nodes reboot into Maintenance Mode (or PXE/ISO boot state).
- Confirm `./talos/` directory is empty.
