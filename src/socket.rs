use std::io;
use std::net::SocketAddr;

use crate::ffi;

/// A cross-platform socket wrapper.
pub struct Socket {
    inner: *mut ffi::Socket,
}

impl Socket {
    /// Initializes the socket subsystem (required on Windows, no-op on Unix).
    pub fn initialize() {
        unsafe { ffi::socket_initialize() };
    }

    /// Cleans up the socket subsystem.
    pub fn cleanup() {
        unsafe { ffi::socket_cleanup() };
    }

    /// Creates a new TCP socket.
    pub fn tcp() -> io::Result<Self> {
        let ptr = unsafe {
            ffi::socket_create(
                libc::AF_INET,
                libc::SOCK_STREAM,
                0,
            )
        };
        if ptr.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Socket { inner: ptr })
        }
    }

    /// Creates a new UDP socket.
    pub fn udp() -> io::Result<Self> {
        let ptr = unsafe {
            ffi::socket_create(
                libc::AF_INET,
                libc::SOCK_DGRAM,
                0,
            )
        };
        if ptr.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Socket { inner: ptr })
        }
    }

    /// Creates a socket with custom domain, type, and protocol.
    pub fn create(domain: i32, socket_type: i32, protocol: i32) -> io::Result<Self> {
        let ptr = unsafe { ffi::socket_create(domain, socket_type, protocol) };
        if ptr.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Socket { inner: ptr })
        }
    }

    /// Binds the socket to the given address.
    pub fn bind(&self, addr: &SocketAddr) -> io::Result<()> {
        let (sockaddr, len) = to_sockaddr(addr);
        let ret = unsafe {
            ffi::socket_bind(
                self.inner,
                &sockaddr as *const libc::sockaddr_in as *const _,
                len,
            )
        };
        if ret == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
    }

    /// Starts listening for connections.
    pub fn listen(&self, backlog: i32) -> io::Result<()> {
        let ret = unsafe { ffi::socket_listen(self.inner, backlog) };
        if ret == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
    }

    /// Accepts an incoming connection.
    pub fn accept(&self) -> io::Result<Socket> {
        let ptr = unsafe {
            ffi::socket_accept(self.inner, std::ptr::null_mut(), std::ptr::null_mut())
        };
        if ptr.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Socket { inner: ptr })
        }
    }

    /// Connects to a remote address.
    pub fn connect(&self, addr: &SocketAddr) -> io::Result<()> {
        let (sockaddr, len) = to_sockaddr(addr);
        let ret = unsafe {
            ffi::socket_connect(
                self.inner,
                &sockaddr as *const libc::sockaddr_in as *const _,
                len,
            )
        };
        if ret == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
    }

    /// Sends data on the socket.
    pub fn send(&self, data: &[u8]) -> io::Result<usize> {
        let ret = unsafe {
            ffi::socket_send(self.inner, data.as_ptr() as *const _, data.len(), 0)
        };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }

    /// Receives data from the socket.
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe {
            ffi::socket_recv(self.inner, buf.as_mut_ptr() as *mut _, buf.len(), 0)
        };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }

    /// Enables SO_REUSEPORT.
    pub fn set_reuse_port(&self, enable: bool) -> io::Result<()> {
        let ret = unsafe { ffi::socket_reuse_port(self.inner, enable as i32) };
        if ret == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
    }

    /// Returns the raw file descriptor.
    pub fn fd(&self) -> i32 {
        unsafe { ffi::socket_fd(self.inner) }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::socket_close(self.inner) };
        }
    }
}

unsafe impl Send for Socket {}

fn to_sockaddr(addr: &SocketAddr) -> (libc::sockaddr_in, u32) {
    match addr {
        SocketAddr::V4(v4) => {
            let mut sa: libc::sockaddr_in = unsafe { std::mem::zeroed() };
            sa.sin_family = libc::AF_INET as u16;
            sa.sin_port = v4.port().to_be();
            sa.sin_addr.s_addr = u32::from_ne_bytes(v4.ip().octets());
            (sa, std::mem::size_of::<libc::sockaddr_in>() as u32)
        }
        SocketAddr::V6(_) => {
            panic!("IPv6 not supported by solidc socket wrapper");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};

    #[test]
    fn test_create_tcp_socket() {
        Socket::initialize();
        let sock = Socket::tcp();
        assert!(sock.is_ok());
        let sock = sock.unwrap();
        assert!(sock.fd() >= 0);
    }

    #[test]
    fn test_create_udp_socket() {
        Socket::initialize();
        let sock = Socket::udp();
        assert!(sock.is_ok());
    }

    #[test]
    fn test_bind_listen() {
        Socket::initialize();
        let sock = Socket::tcp().unwrap();
        sock.set_reuse_port(true).ok();
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));
        assert!(sock.bind(&addr).is_ok());
        assert!(sock.listen(5).is_ok());
    }
}
