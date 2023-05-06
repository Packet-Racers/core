use async_trait::async_trait;
use std::io;
use std::net::SocketAddr;
use tokio::net::{lookup_host, ToSocketAddrs, UdpSocket};

use super::Protocol;

pub struct Udp {
  socket: UdpSocket,
  remote_addr: SocketAddr,
}

impl Udp {
  pub async fn new<A: ToSocketAddrs>(addr: A) -> Result<Self, io::Error> {
    let remote_addr = lookup_host(addr).await?.next().expect("no remote address");
    let socket = UdpSocket::bind(remote_addr).await.map_err(|e| {
      log::debug!("Error binding UDP socket: {}", e);
      e
    })?;

    Ok(Self {
      socket,
      remote_addr,
    })
  }
}

#[async_trait]
impl Protocol for Udp {
  async fn send(&mut self, packet: &[u8]) -> Result<usize, io::Error> {
    self.socket.send_to(packet, &self.remote_addr).await
  }

  async fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, io::Error> {
    let (len, _src_addr) = self.socket.recv_from(buffer).await?;
    Ok(len)
  }

  fn name(&self) -> &str {
    "udp"
  }
}
