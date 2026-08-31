# JARVIS Production Architecture Audit

## 1. Current Privilege Boundaries
The JARVIS architecture is fundamentally divided into two spaces:
- **Unprivileged Zone**: The `jarvis` CLI and TUI run entirely as the invoking user without requiring `sudo` or elevation. They parse configuration, manage unprivileged macros, read general `procfs` state (where permitted), and render the UI.
- **Privileged Zone**: The `jarvis-daemon` service executes privileged kernel interactions. 
- **Trust Boundary**: The Unix Domain Socket (`jarvis.sock`) forms the strict trust boundary.

## 2. IPC Trust Boundaries
The CLI serializes intents into strongly-typed `DaemonRequest` JSON structs. The daemon reads these payloads via Unix Socket.
**Validation Layer**: The daemon calls `libc::getsockopt` with `SO_PEERCRED` to guarantee the peer connecting to the IPC channel is authorized (currently restricted only to root, which breaks proper privilege separation by forcing the client to also be root).

## 3. Root-Required Operations
The daemon strictly requires `root` execution for the following internal actions:
- **cgroup v2 Limitations (`fs::write`)**: Direct filesystem writes to `/sys/fs/cgroup/` endpoints (such as `cgroup.procs`, `cpu.max`, `memory.max`) natively require root permissions or complex pre-configured subtree delegations which are unsuitable for ad-hoc daemon limits.
- **Service Management (`systemctl`)**: Issuing restart/stop commands for arbitrary system-wide services requires root access (unless relying on fragile Polkit rules for every targeted service).
- **Firewall Controls (`ufw`)**: Native configuration of Uncomplicated Firewall underlying tables requires root.
- **Cross-user Process Termination (`kill`)**: `CAP_KILL` is required to terminate processes owned by other users.
- **Cross-user Network Observation**: Enumerating socket inodes under `/proc/<pid>/fd` for external users inherently requires `CAP_SYS_PTRACE` and `CAP_DAC_READ_SEARCH`.

## 4. Operations Not Requiring Root
- Rendering UI.
- Evaluating macros (the macro resolution occurs client-side, the daemon only evaluates single targeted operations).
- Parsing shell commands.
- Discovering current user processes.

## 5. Current Daemon Attack Surface
- **Socket Permissions**: The socket is set to `0660`.
- **Message Parsing**: Validated strongly-typed JSON.
- **Arbitrary Shell Execution**: **Zero.** There are no `sh -c` invocations inside the daemon.
- **Threat Vector**: An attacker intercepting the IPC socket could ask JARVIS to disable `ufw` or stop a system service. Authorization must strictly gate IPC.

## 6. Socket Ownership and Permission Model (Pre-Fix)
Currently, `jarvis-daemon` expects to run as root and conditionally hardcodes `/var/run/jarvis` in its execution path, but validates `peer_uid == 0`. Unprivileged users (e.g. UID 1000) are blocked by the daemon natively.

## 7. Privilege Escalation Paths
- If `jarvis-daemon` ran un-sandboxed as root, and an attacker discovered a path traversal bug in the `target` parameter (e.g. `limit ../../../etc/shadow cpu 50`), they could theoretically manipulate filesystem state. 
- **Mitigation:** The daemon handles typed requests and `fs::write` is restricted directly to `format!("/sys/fs/cgroup/jarvis/processes/{}", pid)`.
- **Systemd Hardening:** We will wrap the daemon in `ProtectSystem=strict` and `ProtectHome=yes` to guarantee the daemon physically cannot alter root filesystems outside of explicitly whitelisted paths.
