# JARVIS v1.0 Release Installation Audit

## 1. Installation Architecture
The JARVIS installation moves from a Cargo-based development workspace to a standard, independent Linux deployment. It provides two distribution models:
- **Bash Installer (`install.sh`)**: Compiles from source and installs binaries, systemd units, and configures environments.
- **Debian Package (`.deb`)**: A pre-compiled `jarvis_1.0.0_amd64.deb` bundle managed natively by `dpkg`/`apt`.

## 2. Filesystem Layout
- **CLI Binary**: `/usr/bin/jarvis` (0755)
- **Daemon Binary**: `/usr/bin/jarvis-daemon` (0755)
- **Daemon Configuration**: `/etc/jarvis/daemon.env` (0644)
- **Runtime Socket Directory**: `/run/jarvis/` (0755, dynamically managed by systemd)
- **systemd Unit**: `/usr/lib/systemd/system/jarvis-daemon.service` (0644)

## 3. Permission Model
- Executables are owned by `root:root` and world-executable (`0755`).
- The daemon service configuration (`/etc/jarvis/daemon.env`) is readable but not writable by unprivileged users.
- The IPC socket `jarvis.sock` is created by the `root` daemon and owned by the `JARVIS_AUTHORIZED_UID` extracted from the `.env` file, allowing only the authorized user to communicate with the daemon.

## 4. systemd Model
- **RuntimeDirectory**: `jarvis` handles the automatic lifecycle of `/run/jarvis`.
- **Environment**: Authorization UID is injected dynamically via `EnvironmentFile=-/etc/jarvis/daemon.env`.
- **Security Sandboxing**: Remained unmodified (`ProtectSystem=strict`, `PrivateTmp=yes`, `NoNewPrivileges=true`, `CapabilityBoundingSet=...`).

## 5. IPC Authorization Model
The authorized user is no longer hardcoded in the `.service` file. During installation, the installers (`install.sh` and `postinst`) derive the authorized `UID` dynamically from the environment (`$SUDO_UID` or a generic 1000 user fallback) and store it in `/etc/jarvis/daemon.env`. The daemon uses this to validate incoming peer credentials over the UNIX socket.

## 6. Installation Verification
The `.deb` packaging script cleanly generates the expected metadata and directory structures without development artifacts. 
`dpkg-deb -c` verification confirmed binaries, service units, and package scripts correctly map to the target layout.

## 7. Uninstallation Verification
The `uninstall.sh` and Debian `prerm` scripts correctly clean up the `/usr/bin/` paths, disable the daemon, and remove the systemd unit without implicitly destroying user-defined configurations unless prompted.

## 8. Package Verification
- Built using native `dpkg-deb`.
- Features fully automated `postinst` (which safely generates `/etc/jarvis/daemon.env` and loads systemd) and `prerm` (which safely shuts down the service).

## 9. Known Environmental Limitations
The current development environment requires interactive `sudo` password authentication. As a result, the physical `sudo ./install.sh` testing to launch the live systemd daemon and test end-to-end unprivileged IPC could not be completed automatically. Verification of the `jarvis-daemon` startup and the actual socket IPC was limited to statically reviewing the package layout and unit structures. 

## 10. Final Release Verdict
**CONDITIONAL RELEASE READY**

JARVIS's layout, installer, package builder, and systemd definitions are complete. It is conditionally ready for release pending a manual `dpkg -i` installation verification on a clean, physical Ubuntu machine where the daemon can be cleanly launched to ensure `EnvironmentFile` correctly injects the UID without breaking peer credential validation.
