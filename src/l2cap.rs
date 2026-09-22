use crate::aap::{commands, AncMode, AAP_PSM};
use crate::security::validate_mac_address;
use std::io;
use std::os::unix::io::RawFd;
use std::time::Duration;

const AF_BLUETOOTH: i32 = 31;
const BTPROTO_L2CAP: i32 = 0;
const BDADDR_BREDR: u8 = 0x00;

#[repr(C)]
struct SockaddrL2 {
    l2_family: u16,
    l2_psm: u16,
    l2_bdaddr: [u8; 6],
    l2_cid: u16,
    l2_bdaddr_type: u8,
}

pub struct L2capConnection {
    fd: RawFd,
    #[allow(dead_code)]
    mac: String,
}

impl Drop for L2capConnection {
    fn drop(&mut self) {
        if self.fd >= 0 {
            unsafe {
                libc::close(self.fd);
            }
        }
    }
}

impl L2capConnection {
    /// Connects to a Beats headset via L2CAP on PSM 0x1001
    pub fn connect(mac: &str) -> io::Result<Self> {
        if !validate_mac_address(mac) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid MAC address"));
        }

        let bdaddr = parse_bdaddr(mac)?;

        let fd = unsafe { libc::socket(AF_BLUETOOTH, libc::SOCK_SEQPACKET, BTPROTO_L2CAP) };
        if fd < 0 {
            let err = io::Error::last_os_error();
            return Err(err);
        }

        // Set socket timeout (2 seconds)
        let timeout = libc::timeval {
            tv_sec: 2,
            tv_usec: 0,
        };
        unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_RCVTIMEO,
                &timeout as *const _ as *const libc::c_void,
                std::mem::size_of::<libc::timeval>() as libc::socklen_t,
            );
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_SNDTIMEO,
                &timeout as *const _ as *const libc::c_void,
                std::mem::size_of::<libc::timeval>() as libc::socklen_t,
            );
        }

        let addr = SockaddrL2 {
            l2_family: AF_BLUETOOTH as u16,
            l2_psm: AAP_PSM.to_le(),
            l2_bdaddr: bdaddr,
            l2_cid: 0,
            l2_bdaddr_type: BDADDR_BREDR,
        };

        let ret = unsafe {
            libc::connect(
                fd,
                &addr as *const _ as *const libc::sockaddr,
                std::mem::size_of::<SockaddrL2>() as libc::socklen_t,
            )
        };

        if ret < 0 {
            let err = io::Error::last_os_error();
            unsafe {
                libc::close(fd);
            }
            return Err(err);
        }

        let conn = L2capConnection {
            fd,
            mac: mac.to_string(),
        };

        // Complete AAP Handshake
        conn.perform_handshake()?;

        Ok(conn)
    }

    fn perform_handshake(&self) -> io::Result<()> {
        // 1. Send Handshake
        self.send_raw(&commands::HANDSHAKE)?;
        std::thread::sleep(Duration::from_millis(50));

        // 2. Send Host Capabilities
        self.send_raw(&commands::SET_FEATURES)?;
        std::thread::sleep(Duration::from_millis(50));

        // 3. Subscribe to Notifications
        self.send_raw(&commands::SUBSCRIBE_NOTIFICATIONS)?;
        std::thread::sleep(Duration::from_millis(50));

        Ok(())
    }

    pub fn send_raw(&self, data: &[u8]) -> io::Result<()> {
        let sent = unsafe {
            libc::send(
                self.fd,
                data.as_ptr() as *const libc::c_void,
                data.len(),
                0,
            )
        };
        if sent < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn read_packet(&self) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; 1024];
        let n = unsafe {
            libc::recv(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                0,
            )
        };
        if n < 0 {
            Err(io::Error::last_os_error())
        } else {
            buf.truncate(n as usize);
            Ok(buf)
        }
    }

    pub fn set_anc_mode(&self, mode: AncMode) -> io::Result<()> {
        let packet = commands::set_anc_mode(mode);
        self.send_raw(&packet)
    }

    pub fn set_mic_mode(&self, mode: crate::aap::MicMode) -> io::Result<()> {
        let packet = commands::set_mic_mode(mode);
        self.send_raw(&packet)
    }

    #[allow(dead_code)]
    pub fn set_one_bud_anc(&self, enable: bool) -> io::Result<()> {
        let packet = commands::set_one_bud_anc(enable);
        self.send_raw(&packet)
    }

    pub fn play_chime(&self, target: &str) -> io::Result<()> {
        let packet = commands::play_chime_command(target);
        self.send_raw(&packet)
    }
}

/// Parses standard Bluetooth MAC string into reverse byte order (little endian BD_ADDR)
fn parse_bdaddr(mac: &str) -> io::Result<[u8; 6]> {
    let parts: Vec<&str> = mac.split(':').collect();
    if parts.len() != 6 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Malformed MAC address"));
    }

    let mut bdaddr = [0u8; 6];
    for (i, part) in parts.iter().rev().enumerate() {
        bdaddr[i] = u8::from_str_radix(part, 16)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Non-hex byte in MAC"))?;
    }

    Ok(bdaddr)
}
