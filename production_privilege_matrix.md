# JARVIS Production Privilege Matrix

## Required Privileges

| Operation | Required Privilege | Current Mechanism | Safer Alternative | Final Decision |
| :--- | :--- | :--- | :--- | :--- |
| **cgroup v2 (limit/unlimit)** | `root` or `CAP_SYS_ADMIN` with cgroup delegation | Runs as `root` and writes to `/sys/fs/cgroup/` | Non-root daemon using systemd DBus to create transient units, OR root daemon tightly sandboxed via systemd `ProtectSystem`. | **Root Daemon with Systemd Sandbox.** Native `fs::write` is structurally safer and faster than DBus transient unit mapping. Wrapping the daemon in a strict read-only filesystem sandbox prevents abuse. |
| **systemctl (start/stop/restart)** | `root` or `Polkit` | Shells out to `systemctl` as `root` | Polkit rules for `jarvis` user to restart specific services. | **Root Daemon.** Generating dynamic Polkit rules for arbitrary services is complex and fragile. The daemon will remain root but restricted via `CapabilityBoundingSet` and IPC validation. |
| **ufw (block/allow)** | `root` | Shells out to `ufw` as `root` | Use `CAP_NET_ADMIN` on an unprivileged daemon. | **Root Daemon.** `ufw` interacts with `/etc/ufw/` files and iptables requiring full root access rather than just capabilities. |
| **Cross-user Process Kill** | `CAP_KILL` | Runs as `root` | Assign `CAP_KILL` to unprivileged daemon binary. | **Root Daemon.** Since `ufw` and `systemctl` demand root, running the daemon as root natively covers this without adding arbitrary capabilities to a non-root binary. |
| **Cross-user Network Procfs** | `CAP_SYS_PTRACE` | Runs as `root` | Drop privileges and run with `CAP_SYS_PTRACE`. | **Root Daemon.** (Same rationale as above). |

## Systemd Sandbox Matrix

To mitigate the risk of running as `root`, the following systemd hardening options will be applied to the `jarvis-daemon.service`:

| Hardening Flag | Purpose | Validation |
| :--- | :--- | :--- |
| `ProtectSystem=strict` | Mounts `/usr`, `/boot`, and `/etc` read-only. | Validated. JARVIS does not write to these paths (except for `/etc/ufw/` which we will whitelist). |
| `ProtectHome=yes` | Prevents the daemon from reading `/home`. | Validated. Daemon state is fully in memory or `/run/jarvis`. |
| `PrivateTmp=yes` | Isolates `/tmp`. | Validated. Daemon does not share temp files. |
| `NoNewPrivileges=true` | Prevents privilege escalation via SUID binaries. | Validated. |
| `ReadWritePaths=/sys/fs/cgroup /run/jarvis /etc/ufw` | Explicitly permits writes to required control boundaries. | Required for cgroup management and IPC socket lifecycle. |
| `CapabilityBoundingSet=` | Restricts maximum capabilities the daemon can obtain. | We will retain `CAP_SYS_ADMIN` (for cgroups), `CAP_NET_ADMIN` (firewall), `CAP_KILL` (proc), `CAP_DAC_OVERRIDE` (ufw configs). |
