use anyhow::Result;
use jarvis_core::cmdlang::ActionRegistry;
use jarvis_core::daemon::{DaemonRequest, DaemonResponse};
use log::{error, info};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

fn get_peer_uid(stream: &UnixStream) -> Result<u32> {
    let mut cred: libc::ucred = unsafe { std::mem::zeroed() };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let res = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut cred as *mut _ as *mut libc::c_void,
            &mut len,
        )
    };
    if res == 0 {
        Ok(cred.uid)
    } else {
        Err(anyhow::anyhow!("Failed to get peer credentials"))
    }
}

fn execute_request(
    req: DaemonRequest,
    registry: &ActionRegistry,
) -> Result<jarvis_core::types::ActionResult> {
    let mut args: Vec<String> = Vec::new();
    let action_str = match req {
        DaemonRequest::StartService { target, force } => {
            if force {
                args.push("--force".to_string());
            }
            args.push(target);
            "start"
        }
        DaemonRequest::StopService { target, force } => {
            if force {
                args.push("--force".to_string());
            }
            args.push(target);
            "stop"
        }
        DaemonRequest::RestartService { target, force } => {
            if force {
                args.push("--force".to_string());
            }
            args.push(target);
            "restart"
        }
        DaemonRequest::EnableService { target, force } => {
            if force {
                args.push("--force".to_string());
            }
            args.push(target);
            "enable"
        }
        DaemonRequest::DisableService { target, force } => {
            if force {
                args.push("--force".to_string());
            }
            args.push(target);
            "disable"
        }
        DaemonRequest::ApplyCgroupLimit {
            target,
            resource,
            value,
            force,
        } => {
            if force {
                args.push("--force".to_string());
            }
            args.push(target);
            args.push(resource);
            args.push(value);
            "limit"
        }
        DaemonRequest::RemoveCgroupLimit { target, force } => {
            if force {
                args.push("--force".to_string());
            }
            args.push(target);
            "unlimit"
        }
        DaemonRequest::NetworkBlock { target, force } => {
            if force {
                args.push("--force".to_string());
            }
            args.push("block".to_string());
            args.push(target);
            "net"
        }
        DaemonRequest::NetworkAllow { target, force } => {
            if force {
                args.push("--force".to_string());
            }
            args.push("allow".to_string());
            args.push(target);
            "net"
        }
    };

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    registry.execute(action_str, &arg_refs)
}

fn handle_client(mut stream: UnixStream, registry: &ActionRegistry) {
    // Peer validation
    match get_peer_uid(&stream) {
        Ok(uid) => {
            // For JARVIS, we expect either root (0) or the intended user (e.g. 1000).
            // This is a basic authorization mechanism.
            let my_uid = unsafe { libc::getuid() };
            if uid != 0 && uid != my_uid {
                error!("Unauthorized connection attempt from UID {}", uid);
                return;
            }
        }
        Err(e) => {
            error!("Could not validate peer: {}", e);
            return;
        }
    }

    let mut reader = BufReader::new(&mut stream);
    let mut request_str = String::new();

    if let Err(e) = reader.read_line(&mut request_str) {
        error!("Failed to read from client: {}", e);
        return;
    }

    let request_str = request_str.trim();
    if request_str.is_empty() {
        return;
    }

    let response = match serde_json::from_str::<DaemonRequest>(request_str) {
        Ok(req) => {
            info!("Received request: {:?}", req);
            match execute_request(req, registry) {
                Ok(result) => DaemonResponse::Success(result),
                Err(e) => DaemonResponse::Error(e.to_string()),
            }
        }
        Err(e) => DaemonResponse::Error(format!("Invalid request format: {}", e)),
    };

    let response_str = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
    if let Err(e) = stream.write_all(response_str.as_bytes()) {
        error!("Failed to write to client: {}", e);
    }
    let _ = stream.write_all(b"\n");
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Use /var/run/jarvis if root, else use ~/.jarvis/daemon.sock for non-root testing.
    let socket_dir = if unsafe { libc::geteuid() } == 0 {
        "/var/run/jarvis".to_string()
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        format!("{}/.jarvis", home)
    };

    if !Path::new(&socket_dir).exists() {
        fs::create_dir_all(&socket_dir)?;
    }

    let socket_path = format!("{}/jarvis.sock", socket_dir);

    if Path::new(&socket_path).exists() {
        fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;

    // Set socket permissions to 0660
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o660))?;

    info!("JARVIS Privileged Daemon listening on {}", socket_path);

    let mut registry = ActionRegistry::new();
    jarvis_core::proc::register_all(&mut registry);
    jarvis_core::svc::register_all(&mut registry);
    jarvis_core::resources::register_all(&mut registry);
    jarvis_core::cgroup::register_all(&mut registry);
    jarvis_core::net::register_all(&mut registry);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream, &registry);
            }
            Err(e) => {
                error!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}
