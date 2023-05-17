use async_trait::async_trait;
use std::{io, sync::Arc};
use std::net::SocketAddr;
use tokio::net::{lookup_host, ToSocketAddrs, UdpSocket};

use super::Protocol;

pub struct Udp {
  socket: Arc<UdpSocket>,
  remote_addr: SocketAddr,
}

impl Udp {
  pub async fn new<A: ToSocketAddrs>(addr: A, socket: Arc<UdpSocket>) -> Result<Self, io::Error> {
    let remote_addr = lookup_host(addr).await?.next().expect("no remote address");
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
