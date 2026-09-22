use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Maximum allowable size for status and cache files (1 MiB).
pub const MAX_FILE_SIZE: u64 = 1024 * 1024;

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
