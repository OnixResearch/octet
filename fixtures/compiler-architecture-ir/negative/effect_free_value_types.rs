use std::fs::Metadata;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::atomic::Ordering;
use std::thread::ThreadId;
use std::time::Duration;

pub fn same_host(left: IpAddr, right: IpAddr) -> bool {
    left == right
}

pub fn loopback(address: Ipv4Addr) -> bool {
    address.is_loopback()
}

pub fn port(address: SocketAddr) -> u16 {
    address.port()
}

pub fn strongest(order: Ordering) -> bool {
    order == Ordering::SeqCst
}

pub fn same_thread(left: ThreadId, right: ThreadId) -> bool {
    left == right
}

pub fn exited(status: ExitStatus) -> bool {
    status.success()
}

pub fn directory(metadata: &Metadata) -> bool {
    metadata.is_dir()
}

pub fn doubled(timeout: Duration) -> Duration {
    timeout * 2
}

pub fn joined(base: &Path) -> PathBuf {
    base.join("child")
}

pub fn connect(address: SocketAddr) -> std::io::Result<TcpStream> {
    TcpStream::connect(address)
}

pub fn read(path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}
