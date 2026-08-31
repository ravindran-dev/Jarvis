# JARVIS Production Capability Audit

## Overview
Due to strict environment limitations preventing passwordless `sudo` or `root` execution, a live runtime capability minimization audit (iteratively removing and verifying systemd `CapabilityBoundingSet` limits) could not be physically performed. 

Instead, this document provides the structural proof for the capabilities currently assigned in the JARVIS production service sandbox.

## Required Capabilities

| Capability | Target Operation | Structural Evidence | Required? |
|---|---|---|---|
| `CAP_SYS_ADMIN` | cgroup v2 modifications | JARVIS directly creates directories via `fs::create_dir_all` and writes limits into `/sys/fs/cgroup/jarvis`. By Linux design, this unified hierarchy is owned by root, and subtree delegation without `CAP_SYS_ADMIN` is only viable via complex transient DBus unit mappings which JARVIS intentionally avoids for architectural simplicity. | **Yes** (Strictly required for native cgroup v2 manipulation). |
| `CAP_NET_ADMIN` | UFW Firewall Control | JARVIS executes `ufw block`/`allow`. `ufw` inherently requires modifying `iptables`/`nftables` which are gated behind `CAP_NET_ADMIN`. | **Yes** (Required for firewall rules). |
| `CAP_KILL` | Terminating Cross-User Processes | JARVIS allows administrators to manage and terminate processes regardless of the owning user. Terminating processes owned by different UIDs requires `CAP_KILL`. | **Yes** (Required for global process control). |
| `CAP_DAC_OVERRIDE` | Reading restricted `procfs` | JARVIS requires enumerating `/proc/<pid>/fd` to correlate network sockets to their owning PIDs. If the PID is owned by another user, standard Discretionary Access Control (DAC) blocks read access to the file descriptors. | **Yes** (Required for global TUI network observation). |

## Capabilities Excluded
By explicitly defining `CapabilityBoundingSet=CAP_SYS_ADMIN CAP_NET_ADMIN CAP_KILL CAP_DAC_OVERRIDE`, we aggressively strip dozens of other root capabilities from the daemon, including:
- `CAP_CHOWN`
- `CAP_SYS_MODULE` (Preventing the daemon from loading kernel modules)
- `CAP_SYS_BOOT`
- `CAP_NET_RAW`
- `CAP_SYS_PTRACE` (We rely on DAC_OVERRIDE instead of PTRACE for procfs, narrowing the attack surface).
