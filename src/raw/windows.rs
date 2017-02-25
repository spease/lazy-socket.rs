use std::io;
use std::os::raw::*;
use std::net;
use std::mem;
use std::cmp;
use std::ptr;
use std::sync::{Once, ONCE_INIT};

//WinAPI Start
mod winapi {
    #![allow(bad_style)]
    #![allow(dead_code)]

    use std::os::raw::*;

    pub type SOCKET = ::std::os::windows::io::RawSocket;
    pub type DWORD = c_ulong;
    pub type WORD = c_ushort;
    pub type GROUP = c_uint;
    pub type CHAR = c_char;
    pub type USHORT = c_ushort;
    pub type ADDRESS_FAMILY = USHORT;
    pub const INVALID_SOCKET: SOCKET = !0;
    pub const SOCKET_ERROR: c_int = -1;
    pub const AF_INET: c_int = 2;
    pub const AF_INET6: c_int = 23;
    pub const WSAESHUTDOWN: DWORD = 10058;
    pub const FD_SETSIZE: usize = 64;

    pub const WSADESCRIPTION_LEN: usize = 256;
    pub const WSASYS_STATUS_LEN: usize = 128;
    #[repr(C)] #[derive(Copy)]
    pub struct WSADATA {
        pub wVersion: WORD,
        pub wHighVersion: WORD,
        #[cfg(target_arch="x86")]
        pub szDescription: [c_char; WSADESCRIPTION_LEN + 1],
        #[cfg(target_arch="x86")]
        pub szSystemStatus: [c_char; WSASYS_STATUS_LEN + 1],
        pub iMaxSockets: c_ushort,
        pub iMaxUdpDg: c_ushort,
        pub lpVendorInfo: *mut c_char,
        #[cfg(target_arch="x86_64")]
        pub szDescription: [c_char; WSADESCRIPTION_LEN + 1],
        #[cfg(target_arch="x86_64")]
        pub szSystemStatus: [c_char; WSASYS_STATUS_LEN + 1],
    }

    #[repr(C)] #[derive(Copy)]
    pub struct FD_SET {
        pub fd_count: c_uint,
        pub fd_array: [SOCKET; FD_SETSIZE],
    }

    impl Clone for FD_SET {
        fn clone(&self) -> FD_SET { *self }
    }

    impl Clone for WSADATA {
        fn clone(&self) -> WSADATA { *self }
    }

    #[repr(C)] #[derive(Clone, Copy)]
    pub struct timeval {
        pub tv_sec: c_long,
        pub tv_usec: c_long,
    }

    #[repr(C)]
    pub struct SOCKADDR_STORAGE_LH {
        pub ss_family: ADDRESS_FAMILY,
        pub __ss_pad1: [CHAR; 6],
        pub __ss_align: i64,
        pub __ss_pad2: [CHAR; 112],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct in_addr {
        pub s_addr: [u8; 4],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct in6_addr {
        pub s6_addr: [u16; 8],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct sockaddr_in {
        pub sin_family: ADDRESS_FAMILY,
        pub sin_port: USHORT,
        pub sin_addr: in_addr,
        pub sin_zero: [CHAR; 8],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct sockaddr_in6 {
        pub sin6_family: ADDRESS_FAMILY,
        pub sin6_port: USHORT,
        pub sin6_flowinfo: c_ulong,
        pub sin6_addr: in6_addr,
        pub sin6_scope_id: c_ulong,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct SOCKADDR {
        pub sa_family: ADDRESS_FAMILY,
        pub sa_data: [CHAR; 14],
    }

    pub type LPWSADATA = *mut WSADATA;

    extern "system" {
        pub fn WSAStartup(wVersionRequested: WORD, lpWSAData: LPWSADATA) -> c_int;
        pub fn WSACleanup() -> c_int;

        pub fn getsockname(s: SOCKET, name: *mut SOCKADDR, namelen: *mut c_int) -> c_int;
        pub fn socket(af: c_int, _type: c_int, protocol: c_int) -> SOCKET;
        pub fn bind(s: SOCKET, name: *const SOCKADDR, namelen: c_int) -> c_int;
        pub fn listen(s: SOCKET, backlog: c_int) -> c_int;
        pub fn accept(s: SOCKET, addr: *mut SOCKADDR, addrlen: *mut c_int) -> SOCKET;
        pub fn connect(s: SOCKET, name: *const SOCKADDR, namelen: c_int) -> c_int;
        pub fn recv(s: SOCKET, buf: *mut c_char, len: c_int, flags: c_int) -> c_int;
        pub fn recvfrom(s: SOCKET, buf: *mut c_char, len: c_int, flags: c_int, from: *mut SOCKADDR, fromlen: *mut c_int) -> c_int;
        pub fn send(s: SOCKET, buf: *const c_char, len: c_int, flags: c_int) -> c_int;
        pub fn sendto(s: SOCKET, buf: *const c_char, len: c_int, flags: c_int, to: *const SOCKADDR, tolen: c_int) -> c_int;
        pub fn getsockopt(s: SOCKET, level: c_int, optname: c_int, optval: *mut c_char, optlen: *mut c_int) -> c_int;
        pub fn setsockopt(s: SOCKET, level: c_int, optname: c_int, optval: *const c_char, optlen: c_int) -> c_int;
        pub fn ioctlsocket(s: SOCKET, cmd: c_long, argp: *mut c_ulong) -> c_int;
        pub fn shutdown(s: SOCKET, how: c_int) -> c_int;
        pub fn closesocket(s: SOCKET) -> c_int;
        pub fn select(nfds: c_int, readfds: *mut FD_SET, writefds: *mut FD_SET, exceptfds: *mut FD_SET, timeout: *const timeval) -> c_int;
    }
}

macro_rules! impl_into_trait {
    ($($t:ty), +) => {
        $(
            impl Into<c_int> for $t {
                fn into(self) -> c_int {
                    self as c_int
                }
            }
        )+
    };
}

#[allow(non_snake_case)]
///Socket family
pub mod Family {
    use std::os::raw::c_int;
    pub const UNSPECIFIED: c_int = 0;
    pub const IPV4: c_int = 2;
    pub const IPV6: c_int = 23;
    pub const IRDA: c_int = 26;
    pub const BTH: c_int = 32;
}

#[allow(non_snake_case)]
///Socket type
pub mod Type {
    use std::os::raw::c_int;
    pub const STREAM: c_int = 1;
    pub const DATAGRAM: c_int = 2;
    pub const RAW: c_int = 3;
    pub const RDM: c_int = 4;
    pub const SEQPACKET: c_int = 5;
}

#[allow(non_snake_case)]
///Socket protocol
pub mod Protocol {
    use std::os::raw::c_int;
    pub const NONE: c_int = 0;
    pub const ICMP: c_int = 1;
    pub const TCP: c_int = 6;
    pub const UDP: c_int = 17;
    pub const ICMPV6: c_int = 58;
}

#[derive(Copy, Clone)]
///Type of socket's shutdown operation.
pub enum ShutdownType {
    ///Stops any further receives.
    Receive = 0,
    ///Stops any further sends.
    Send = 1,
    ///Stops both sends and receives.
    Both = 2
}

impl_into_trait!(ShutdownType);

///Raw socket
pub struct Socket {
    inner: winapi::SOCKET
}

impl Socket {
    ///Initializes new socket.
    ///
    ///Corresponds to C connect()
    pub fn new(family: c_int, _type: c_int, protocol: c_int) -> io::Result<Socket> {
        static INIT: Once = ONCE_INIT;

        INIT.call_once(|| {
            //just to initialize winsock inside libstd
            let _ = net::UdpSocket::bind("127.0.0.1:34254");
        });

        unsafe {
            match winapi::socket(family, _type, protocol) {
                winapi::INVALID_SOCKET => Err(io::Error::last_os_error()),
                fd => Ok(Socket {
                    inner: fd
                }),
            }
        }
    }

    ///Returns underlying socket descriptor.
    ///
    ///Note: ownership is not transferred.
    pub fn raw(&self) -> winapi::SOCKET {
        self.inner
    }

    ///Retrieves socket name i.e. address
    ///
    ///Wraps `getsockname()`
    ///
    ///Available for binded/connected sockets.
    pub fn name(&self) -> io::Result<net::SocketAddr> {
        unsafe {
            let mut storage: winapi::SOCKADDR_STORAGE_LH = mem::zeroed();
            let mut len = mem::size_of_val(&storage) as c_int;

            match winapi::getsockname(self.inner, &mut storage as *mut _ as *mut _, &mut len) {
                winapi::SOCKET_ERROR => Err(io::Error::last_os_error()),
                _ => sockaddr_to_addr(&storage, len)
            }
        }
    }

    ///Binds socket to address.
    pub fn bind(&self, addr: &net::SocketAddr) -> io::Result<()> {
        let (addr, len) = get_raw_addr(addr);

        unsafe {
            match winapi::bind(self.inner, addr, len) {
                0 => Ok(()),
                _ => Err(io::Error::last_os_error())
            }
        }
    }

    ///Listens for incoming connections on this socket.
    pub fn listen(&self, backlog: c_int) -> io::Result<()> {
        unsafe {
            match winapi::listen(self.inner, backlog) {
                0 => Ok(()),
                _ => Err(io::Error::last_os_error())
            }
        }
    }

    ///Receives some bytes from socket
    ///
    ///Number of received bytes is returned on success
    pub fn recv(&self, buf: &mut [u8], flags: c_int) -> io::Result<usize> {
        let len = cmp::min(buf.len(), i32::max_value() as usize) as i32;
        unsafe {
            match winapi::recv(self.inner, buf.as_mut_ptr() as *mut c_char, len, flags) {
                -1 => {
                    let error = io::Error::last_os_error();
                    let raw_code = error.raw_os_error().unwrap();

                    if raw_code == winapi::WSAESHUTDOWN as i32 {
                        Ok(0)
                    }
                    else {
                        Err(error)
                    }
                },
                n => Ok(n as usize)
            }
        }
    }

    ///Receives some bytes from socket
    ///
    ///Number of received bytes and remote address are returned on success.
    pub fn recv_from(&self, buf: &mut [u8], flags: c_int) -> io::Result<(usize, net::SocketAddr)> {
        let len = cmp::min(buf.len(), i32::max_value() as usize) as i32;
        unsafe {
            let mut storage: winapi::SOCKADDR_STORAGE_LH = mem::zeroed();
            let mut storage_len = mem::size_of_val(&storage) as c_int;

            match winapi::recvfrom(self.inner, buf.as_mut_ptr() as *mut c_char, len, flags, &mut storage as *mut _ as *mut _, &mut storage_len) {
                -1 => {
                    let error = io::Error::last_os_error();
                    let raw_code = error.raw_os_error().unwrap();

                    if raw_code == winapi::WSAESHUTDOWN as i32 {
                        let peer_addr = sockaddr_to_addr(&storage, storage_len)?;
                        Ok((0, peer_addr))
                    }
                    else {
                        Err(error)
                    }
                },
                n => {
                    let peer_addr = sockaddr_to_addr(&storage, storage_len)?;
                    Ok((n as usize, peer_addr))
                }
            }
        }
    }

    ///Sends some bytes through socket.
    ///
    ///Number of sent bytes is returned.
    pub fn send(&self, buf: &[u8], flags: c_int) -> io::Result<usize> {
        let len = cmp::min(buf.len(), i32::max_value() as usize) as i32;

        unsafe {
            match winapi::send(self.inner, buf.as_ptr() as *const c_char, len, flags) {
                -1 => {
                    let error = io::Error::last_os_error();
                    let raw_code = error.raw_os_error().unwrap();

                    if raw_code == winapi::WSAESHUTDOWN as i32 {
                        Ok(0)
                    }
                    else {
                        Err(error)
                    }
                },
                n => Ok(n as usize)
            }
        }
    }

    ///Sends some bytes through socket toward specified peer.
    ///
    ///Number of sent bytes is returned.
    ///
    ///Note: the socket will be bound, if it isn't already.
    ///Use method `name` to determine address.
    pub fn send_to(&self, buf: &[u8], peer_addr: &net::SocketAddr, flags: c_int) -> io::Result<usize> {
        let len = cmp::min(buf.len(), i32::max_value() as usize) as i32;
        let (addr, addr_len) = get_raw_addr(peer_addr);

        unsafe {
            match winapi::sendto(self.inner, buf.as_ptr() as *const c_char, len, flags, addr, addr_len) {
                -1 => {
                    let error = io::Error::last_os_error();
                    let raw_code = error.raw_os_error().unwrap();

                    if raw_code == winapi::WSAESHUTDOWN as i32 {
                        Ok(0)
                    }
                    else {
                        Err(error)
                    }
                },
                n => Ok(n as usize)
            }
        }
    }

    ///Accepts incoming connection.
    pub fn accept(&self) -> io::Result<(Socket, net::SocketAddr)> {
        unsafe {
            let mut storage: winapi::SOCKADDR_STORAGE_LH = mem::zeroed();
            let mut len = mem::size_of_val(&storage) as c_int;

            match winapi::accept(self.inner, &mut storage as *mut _ as *mut _, &mut len) {
                winapi::INVALID_SOCKET => Err(io::Error::last_os_error()),
                sock @ _ => {
                    let addr = sockaddr_to_addr(&storage, len)?;
                    Ok((Socket { inner: sock, }, addr))
                }
            }
        }
    }

    ///Connects socket with remote address.
    pub fn connect(&self, addr: &net::SocketAddr) -> io::Result<()> {
        let (addr, len) = get_raw_addr(addr);

        unsafe {
            match winapi::connect(self.inner, addr, len) {
                0 => Ok(()),
                _ => Err(io::Error::last_os_error())
            }
        }
    }

    ///Retrieves socket option.
    pub fn get_opt<T>(&self, level: c_int, name: c_int) -> io::Result<T> {
        unsafe {
            let mut value: T = mem::zeroed();
            let value_ptr = &mut value as *mut T as *mut c_char;
            let mut value_len = mem::size_of::<T>() as c_int;

            match winapi::getsockopt(self.inner, level, name, value_ptr, &mut value_len) {
                0 => Ok(value),
                _ => Err(io::Error::last_os_error())
            }
        }
    }

    ///Sets socket option
    ///
    ///Value is generally integer or C struct.
    pub fn set_opt<T>(&self, level: c_int, name: c_int, value: T) -> io::Result<()> {
        unsafe {
            let value = &value as *const T as *const c_char;

            match winapi::setsockopt(self.inner, level, name, value, mem::size_of::<T>() as c_int) {
                0 => Ok(()),
                _ => Err(io::Error::last_os_error())
            }
        }
    }

    ///Sets I/O parameters of socket.
    ///
    ///It uses `ioctlsocket` under hood.
    pub fn ioctl(&self, request: c_int, value: c_ulong) -> io::Result<()> {
        unsafe {
            let mut value = value;
            let value = &mut value as *mut c_ulong;

            match winapi::ioctlsocket(self.inner, request, value) {
                0 => Ok(()),
                _ => Err(io::Error::last_os_error())
            }
        }
    }

    ///Sets non-blocking mode.
    pub fn set_nonblocking(&self, value: bool) -> io::Result<()> {
        const FIONBIO: c_ulong = 0x8004667e;

        self.ioctl(FIONBIO as c_long, value as c_ulong)
    }


    ///Stops receive and/or send over socket.
    pub fn shutdown(&self, direction: ShutdownType) -> io::Result<()> {
        unsafe {
            match winapi::shutdown(self.inner, direction.into()) {
                0 => Ok(()),
                _ => Err(io::Error::last_os_error())
            }
        }
    }

    ///Closes socket.
    ///
    ///Note: on `Drop` socket will be closed on its own.
    ///There is no need to close it explicitly.
    pub fn close(&self) -> io::Result<()> {
        unsafe {
            match winapi::closesocket(self.inner) {
                0 => Ok(()),
                _ => Err(io::Error::last_os_error())
            }
        }
    }
}

fn get_raw_addr(addr: &net::SocketAddr) -> (*const winapi::SOCKADDR, c_int) {
    match *addr {
        net::SocketAddr::V4(ref a) => {
            (a as *const _ as *const _, mem::size_of_val(a) as c_int)
        }
        net::SocketAddr::V6(ref a) => {
            (a as *const _ as *const _, mem::size_of_val(a) as c_int)
        }
    }
}

fn sockaddr_to_addr(storage: &winapi::SOCKADDR_STORAGE_LH, len: c_int) -> io::Result<net::SocketAddr> {
    match storage.ss_family as c_int {
        winapi::AF_INET => {
            assert!(len as usize >= mem::size_of::<winapi::sockaddr_in>());
            let storage = unsafe { *(storage as *const _ as *const winapi::sockaddr_in) };
            let ip = net::Ipv4Addr::new(storage.sin_addr.s_addr[0],
                                        storage.sin_addr.s_addr[1],
                                        storage.sin_addr.s_addr[2],
                                        storage.sin_addr.s_addr[3]);

            //Note to_be() swap bytes on LE targets
            //As IP stuff is always BE, we need swap only on LE targets
            Ok(net::SocketAddr::V4(net::SocketAddrV4::new(ip, storage.sin_port.to_be())))
        }
        winapi::AF_INET6 => {
            assert!(len as usize >= mem::size_of::<winapi::sockaddr_in6>());
            let storage = unsafe { *(storage as *const _ as *const winapi::sockaddr_in6) };
            let ip = net::Ipv6Addr::new(storage.sin6_addr.s6_addr[0],
                                        storage.sin6_addr.s6_addr[1],
                                        storage.sin6_addr.s6_addr[2],
                                        storage.sin6_addr.s6_addr[3],
                                        storage.sin6_addr.s6_addr[4],
                                        storage.sin6_addr.s6_addr[5],
                                        storage.sin6_addr.s6_addr[6],
                                        storage.sin6_addr.s6_addr[7]);

            Ok(net::SocketAddr::V6(net::SocketAddrV6::new(ip, storage.sin6_port.to_be(), storage.sin6_flowinfo, storage.sin6_scope_id)))
        }
        _ => {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid addr type."))
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = self.shutdown(ShutdownType::Both);
        let _ = self.close();
    }
}

use std::os::windows::io::{
    AsRawSocket,
    FromRawSocket,
    IntoRawSocket,
};

impl AsRawSocket for Socket {
    fn as_raw_socket(&self) -> winapi::SOCKET {
        self.inner
    }
}

impl FromRawSocket for Socket {
    unsafe fn from_raw_socket(sock: winapi::SOCKET) -> Self {
        Socket {inner: sock}
    }
}

impl IntoRawSocket for Socket {
    fn into_raw_socket(self) -> winapi::SOCKET {
        let result = self.inner;
        mem::forget(self);
        result
    }
}

#[inline]
fn ms_to_timeval(timeout_ms: u64) -> winapi::timeval {
    winapi::timeval {
        tv_sec: timeout_ms as c_long / 1000,
        tv_usec: (timeout_ms as c_long % 1000) * 1000
    }
}

fn sockets_to_fd_set(sockets: &[&Socket]) -> winapi::FD_SET {
    assert!(sockets.len() < winapi::FD_SETSIZE);
    let mut raw_fds: winapi::FD_SET = unsafe { mem::zeroed() };

    for socket in sockets {
        let idx = raw_fds.fd_count as usize;
        raw_fds.fd_array[idx] = socket.inner;
        raw_fds.fd_count += 1;
    }

    raw_fds
}

///Wrapper over system `select`
///
///Returns number of sockets that are ready.
///
///If timeout isn't specified then select will be blocking call.
///
///## Note:
///
///Number of each set cannot be bigger than FD_SETSIZE i.e. 64
///
///## Warning:
///
///It is invalid to pass all sets of descriptors empty on Windows.
pub fn select(read_fds: &[&Socket], write_fds: &[&Socket], except_fds: &[&Socket], timeout_ms: Option<u64>) -> io::Result<c_int> {
    let mut raw_read_fds = sockets_to_fd_set(read_fds);
    let mut raw_write_fds = sockets_to_fd_set(write_fds);
    let mut raw_except_fds = sockets_to_fd_set(except_fds);

    unsafe {
        match winapi::select(0,
                             if read_fds.len() > 0 { &mut raw_read_fds } else { ptr::null_mut() },
                             if write_fds.len() > 0 { &mut raw_write_fds } else { ptr::null_mut() },
                             if except_fds.len() > 0 { &mut raw_except_fds } else { ptr::null_mut() },
                             if let Some(timeout_ms) = timeout_ms { &ms_to_timeval(timeout_ms) } else { ptr::null() } ) {
            winapi::SOCKET_ERROR => Err(io::Error::last_os_error()),
            result @ _ => Ok(result)

        }
    }
}
