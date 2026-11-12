use tokio::io::{self, AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

/// Buffer por direção. Um tamanho maior reduz syscalls e melhora throughput em
/// downloads/uploads longos, sem criar um buffer gigante por conexão.
const RELAY_BUFFER_SIZE: usize = 128 * 1024;
const SOCKET_BUFFER_SIZE: libc::c_int = 512 * 1024;

/// Ajustes seguros para túneis TCP interativos e transferências contínuas.
///
/// TCP_NODELAY reduz a espera do Nagle em SSH. Buffers maiores ajudam redes
/// móveis com janela TCP maior. TCP_QUICKACK é reaplicado no Linux quando
/// disponível; ele não é exigido para o funcionamento em outros sistemas.
pub fn tune_tcp_stream(stream: &TcpStream) {
    let _ = stream.set_nodelay(true);
    #[cfg(target_os = "linux")]
    unsafe {
        let fd = stream.as_raw_fd();
        let size = std::mem::size_of_val(&SOCKET_BUFFER_SIZE) as libc::socklen_t;
        let _ = libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            &SOCKET_BUFFER_SIZE as *const _ as *const libc::c_void,
            size,
        );
        let _ = libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_SNDBUF,
            &SOCKET_BUFFER_SIZE as *const _ as *const libc::c_void,
            size,
        );
        let quickack: libc::c_int = 1;
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_QUICKACK,
            &quickack as *const _ as *const libc::c_void,
            std::mem::size_of_val(&quickack) as libc::socklen_t,
        );
    }
}

pub async fn copy_bidirectional<A, B>(a: &mut A, b: &mut B) -> io::Result<(u64, u64)>
where
    A: AsyncRead + AsyncWrite + Unpin,
    B: AsyncRead + AsyncWrite + Unpin,
{
    tokio::io::copy_bidirectional_with_sizes(
        a,
        b,
        RELAY_BUFFER_SIZE,
        RELAY_BUFFER_SIZE,
    )
    .await
}

#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
