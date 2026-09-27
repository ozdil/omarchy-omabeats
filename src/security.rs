use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Maximum allowable size for status and cache files (1 MiB).
pub const MAX_FILE_SIZE: u64 = 1024 * 1024;

/// Verifies that the connected peer on a Unix domain socket matches the current process UID.
/// Enforces Zero-Trust local peer authentication (SO_PEERCRED).
pub fn verify_socket_peer_credentials(stream: &UnixStream) -> io::Result<libc::ucred> {
    let fd = stream.as_raw_fd();
    let mut ucred: libc::ucred = unsafe { std::mem::zeroed() };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;

    // SAFETY: getsockopt is called with a valid socket fd and pointer to ucred
    let ret = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut ucred as *mut _ as *mut libc::c_void,
            &mut len,
        )
    };

    if ret != 0 {
        return Err(io::Error::last_os_error());
    }

    let current_uid = unsafe { libc::getuid() };
    if ucred.uid != current_uid {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Zero-Trust Access Denied: Peer UID {} does not match authorized process UID {}",
                ucred.uid, current_uid
            ),
        ));
    }

    Ok(ucred)
}

/// Runs a command with a strict monotonic deadline, process isolation, and guarantees process cleanup.
#[allow(dead_code)]
pub fn run_command_with_deadline(mut cmd: Command, timeout: Duration) -> io::Result<String> {
    cmd.process_group(0);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    let mut child = cmd.spawn()?;
    let pid = child.id() as i32;
    let start = std::time::Instant::now();

    loop {
        if let Ok(Some(status)) = child.try_wait() {
            if !status.success() {
                return Err(io::Error::new(io::ErrorKind::Other, "Command exited with non-zero status"));
            }
            break;
        }

        if start.elapsed() >= timeout {
            // Deadline exceeded - reap immediately
            if pid > 1 {
                unsafe {
                    libc::kill(-pid, libc::SIGTERM);
                }
                std::thread::sleep(Duration::from_millis(15));
                unsafe {
                    libc::kill(-pid, libc::SIGKILL);
                }
            }
            let _ = child.wait();
            return Err(io::Error::new(io::ErrorKind::TimedOut, "Command monotonic deadline exceeded"));
        }

        std::thread::sleep(Duration::from_millis(20));
    }

    let mut stdout_buf = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        stdout.take(MAX_FILE_SIZE).read_to_end(&mut stdout_buf)?;
    }

    String::from_utf8(stdout_buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// RAII ProcessGroupGuard ensuring subprocess groups are reaped upon drop.
pub struct ProcessGroupGuard {
    pub child: Option<Child>,
}

impl ProcessGroupGuard {
    pub fn new(child: Child) -> Self {
        Self { child: Some(child) }
    }

    #[allow(dead_code)]
    pub fn take(&mut self) -> Option<Child> {
        self.child.take()
    }
}

impl Drop for ProcessGroupGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            reap_process_group(&mut child);
        }
    }
}

/// Unconditionally terminates an entire process group (-pid) with SIGTERM, 15ms grace, then SIGKILL.
pub fn reap_process_group(child: &mut Child) {
    let pid = child.id() as i32;
    if pid <= 1 {
        let _ = child.wait();
        return;
    }

    let exited = matches!(child.try_wait(), Ok(Some(_)));
    if !exited {
        // SAFETY: pid is a valid child PID spawned with process_group(0).
        unsafe {
            libc::kill(-pid, libc::SIGTERM);
        }

        std::thread::sleep(Duration::from_millis(15));

        unsafe {
            libc::kill(-pid, libc::SIGKILL);
        }
    }

    let _ = child.wait();
}

/// Spawns a command inside an isolated process group (process_group(0)).
pub fn spawn_isolated(mut cmd: Command) -> io::Result<ProcessGroupGuard> {
    cmd.process_group(0);
    cmd.stdin(Stdio::null());
    let child = cmd.spawn()?;
    Ok(ProcessGroupGuard::new(child))
}

/// Safely reads a file with strict 1 MiB size ceiling and symlink rejection.
pub fn safe_read_file_limited(path: &Path) -> io::Result<String> {
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File does not exist"));
    }

    let meta = fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Symlinks are strictly prohibited"));
    }

    if meta.len() > MAX_FILE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File size exceeds 1 MiB ceiling"));
    }

    let file = File::open(path)?;
    let mut take_reader = file.take(MAX_FILE_SIZE + 1);
    let mut buffer = Vec::new();
    take_reader.read_to_end(&mut buffer)?;

    if buffer.len() as u64 > MAX_FILE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File read exceeded 1 MiB limit"));
    }

    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Atomically writes data to a secure file with mode 0600 and directory mode 0700.
pub fn atomic_write_secure(target: &Path, content: &str) -> io::Result<()> {
    if let Some(parent) = target.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        }
    }

    if target.exists() {
        let meta = fs::symlink_metadata(target)?;
        if meta.file_type().is_symlink() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Cannot overwrite symlink"));
        }
    }

    let filename = target.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let tmp_name = format!(".tmp_{}_{}_{}", filename, std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
    let tmp_path = target.with_file_name(tmp_name);

    {
        let mut tmp_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&tmp_path)?;

        tmp_file.write_all(content.as_bytes())?;
        tmp_file.sync_all()?;
    }

    fs::rename(&tmp_path, target)?;
    Ok(())
}

/// Validates that a string is a valid Bluetooth MAC address (e.g. 04:9D:05:DD:08:62).
pub fn validate_mac_address(mac: &str) -> bool {
    if mac.len() != 17 {
        return false;
    }
    let bytes = mac.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if i % 3 == 2 {
            if b != b':' {
                return false;
            }
        } else {
            if !b.is_ascii_hexdigit() {
                return false;
            }
        }
    }
    true
}

/// Sanitizes input strings by stripping command characters.
#[allow(dead_code)]
pub fn sanitize_alphanumeric(input: &str, max_len: usize) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_' || *c == '.' || *c == ':')
        .take(max_len)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_trust_peer_credentials() {
        let (s1, _s2) = UnixStream::pair().expect("Failed to create UnixStream pair");
        let cred = verify_socket_peer_credentials(&s1).expect("Peer credential check should succeed for same process");
        assert_eq!(cred.uid, unsafe { libc::getuid() });
    }

    #[test]
    fn test_monotonic_deadline_command_success() {
        let mut cmd = Command::new("/bin/echo");
        cmd.arg("omarchy_beats_test");
        let out = run_command_with_deadline(cmd, Duration::from_secs(2)).expect("Command should succeed");
        assert!(out.contains("omarchy_beats_test"));
    }

    #[test]
    fn test_monotonic_deadline_command_timeout() {
        let mut cmd = Command::new("/bin/sleep");
        cmd.arg("5");
        let res = run_command_with_deadline(cmd, Duration::from_millis(100));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), io::ErrorKind::TimedOut);
    }

    #[test]
    fn test_validate_mac_address() {
        assert!(validate_mac_address("04:9D:05:DD:08:62"));
        assert!(!validate_mac_address("04:9D:05:DD:08:6Z"));
        assert!(!validate_mac_address("invalid_mac"));
        assert!(!validate_mac_address("04-9D-05-DD-08-62"));
    }
}

