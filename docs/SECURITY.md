# JARVIS Security and Privilege Model

JARVIS employs a strict privilege separation model to ensure unprivileged users can safely manage system limits and view metrics without escalating privileges.

## Trust Boundary
The core security concept relies on **Unprivileged Client -> Privileged Daemon** architecture over an authorized Unix Domain Socket.
- The `jarvis` client (CLI/TUI) runs entirely unprivileged.
- The `jarvis-daemon` runs as `root` (managed via systemd) to perform strictly bounded Linux kernel actions.

## Daemon Sandboxing
To mitigate the risks of a `root` daemon, `jarvis-daemon.service` is wrapped in a tight systemd sandbox:
- `ProtectSystem=strict`: The entire root filesystem (`/usr`, `/etc`, `/boot`) is mounted read-only.
- `ProtectHome=yes`: The daemon is cryptographically isolated from `/home`.
- `ReadWritePaths=/sys/fs/cgroup /run/jarvis /etc/ufw`: Only explicitly necessary paths are writable.
- `NoNewPrivileges=true`: Disables SUID execution.

## IPC Authentication
The `jarvis.sock` IPC channel requires:
1. **File Permissions**: The socket is set to `0660`.
2. **Kernel Peer Validation**: The daemon actively calls `SO_PEERCRED` to securely verify the UID of the connecting process from the kernel. It rejects any UID that does not explicitly match `root` (0) or the configured `JARVIS_AUTHORIZED_UID`.

## Supported Privileged Operations
The daemon exclusively supports predefined JSON RPC instructions mapping to specific actions. It does **not** evaluate arbitrary shell commands.
- `ApplyCgroupLimit`: Directly creates and writes limits into `/sys/fs/cgroup/jarvis`.
- `RemoveCgroupLimit`: Re-parents PIDs to the root cgroup.
- `NetworkBlock`/`NetworkAllow`: Executes typed `ufw` arguments.
- `RestartService`/`StopService`: Executes typed `systemctl` actions.
