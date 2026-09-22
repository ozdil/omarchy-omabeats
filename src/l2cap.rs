use crate::aap::{commands, AncMode, AAP_PSM};
use crate::security::validate_mac_address;
use std::io;
use std::os::unix::io::RawFd;
use std::sync::Mutex;
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
    pub mac: String,
    send_lock: Mutex<()>,
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
            return Err(io::Error::last_os_error());
        }

        // Set socket send timeout (1 second)
        let timeout = libc::timeval {
            tv_sec: 1,
            tv_usec: 0,
        };
        unsafe {
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
            send_lock: Mutex::new(()),
        };

        // Complete AAP Handshake & Initial Setup
        conn.perform_handshake()?;

        Ok(conn)
    }

    fn perform_handshake(&self) -> io::Result<()> {
        // 1. Send Handshake
        self.send_raw(&commands::HANDSHAKE)?;
        let _ = self.poll_read_packet(100);

        // 2. Send Host Capabilities (iOS features equivalent)
        self.send_raw(&commands::SET_FEATURES)?;
        let _ = self.poll_read_packet(100);

        // 3. Subscribe to Notifications
        self.send_raw(&commands::SUBSCRIBE_NOTIFICATIONS)?;
        std::thread::sleep(Duration::from_millis(50));

        // 4. Enable All Listening Modes in firmware rotation (Off + Noise + Transparency + Adaptive)
        let _ = self.send_raw(&commands::ENABLE_ALL_LISTENING_MODES);
        std::thread::sleep(Duration::from_millis(30));

        // 5. Enable One-Bud ANC so ANC / Transparency works even with one bud
        let _ = self.send_raw(&commands::set_one_bud_anc(true));

        Ok(())
    }

    pub fn send_raw(&self, data: &[u8]) -> io::Result<()> {
        let _guard = self.send_lock.lock().unwrap();
        eprintln!("[l2cap-send] {:02x?}", data);
        let sent = unsafe {
            libc::send(
                self.fd,
                data.as_ptr() as *const libc::c_void,
                data.len(),
                libc::MSG_NOSIGNAL,
            )
        };
        if sent < 0 {
            let err = io::Error::last_os_error();
            eprintln!("[l2cap-send-err] {}", err);
            Err(err)
        } else {
            Ok(())
        }
    }

    /// Polls the L2CAP socket for incoming packets with specified timeout (milliseconds)
    pub fn poll_read_packet(&self, timeout_ms: i32) -> io::Result<Option<Vec<u8>>> {
        let mut pfd = libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN | libc::POLLERR | libc::POLLHUP,
            revents: 0,
        };

        let ret = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }
        if ret == 0 {
            // Timeout, no data pending
            return Ok(None);
        }

        if (pfd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL)) != 0 {
            return Err(io::Error::new(io::ErrorKind::ConnectionReset, "L2CAP socket error or hangup"));
        }

        if (pfd.revents & libc::POLLIN) != 0 {
            let mut buf = vec![0u8; 1024];
            let n = unsafe {
                libc::recv(
                    self.fd,
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                    libc::MSG_DONTWAIT,
                )
            };
            if n < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::WouldBlock {
                    return Ok(None);
                }
                return Err(err);
            } else if n == 0 {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed by remote"));
            } else {
                buf.truncate(n as usize);
                return Ok(Some(buf));
            }
        }

        Ok(None)
    }

    /// Drains any pending buffered packets from the socket immediately without blocking
    pub fn drain(&self) {
        let mut buf = vec![0u8; 1024];
        loop {
            let n = unsafe {
                libc::recv(
                    self.fd,
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                    libc::MSG_DONTWAIT,
                )
            };
            if n <= 0 {
                break;
            }
        }
    }

    /// Reads all available AAP notification packets until timeout or max duration reached
    pub fn read_all_notifications(&self, max_duration: Duration) -> Vec<Vec<u8>> {
        let mut packets = Vec::new();
        let start = std::time::Instant::now();
        while start.elapsed() < max_duration {
            match self.poll_read_packet(50) {
                Ok(Some(data)) => {
                    if !data.is_empty() {
                        packets.push(data);
                    }
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }
        packets
    }

    pub fn set_anc_mode(&self, mode: AncMode) -> io::Result<()> {
        let _ = self.send_raw(&commands::ENABLE_ALL_LISTENING_MODES);
        std::thread::sleep(Duration::from_millis(15));
        let packet = commands::set_anc_mode(mode);
        self.send_raw(&packet)
    }

    pub fn set_mic_mode(&self, mode: crate::aap::MicMode) -> io::Result<()> {
        let packet = commands::set_mic_mode(mode);
        self.send_raw(&packet)
    }

    pub fn set_in_ear_detection(&self, enable: bool) -> io::Result<()> {
        let packet = commands::set_in_ear_detection(enable);
        self.send_raw(&packet)
    }

    #[allow(dead_code)]
    pub fn set_one_bud_anc(&self, enable: bool) -> io::Result<()> {
        let packet = commands::set_one_bud_anc(enable);
        self.send_raw(&packet)
    }

    #[allow(dead_code)]
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
