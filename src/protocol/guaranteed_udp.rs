use async_trait::async_trait;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{lookup_host, ToSocketAddrs, UdpSocket};
use tokio::time::timeout;

use super::Protocol;

const ACK_TIMEOUT: Duration = Duration::from_secs(1);

pub struct GuaranteedUdp {
  socket: UdpSocket,
  remote_addr: SocketAddr,
}

impl GuaranteedUdp {
  pub async fn new<A: ToSocketAddrs>(local_addr: A, remote_addr: A) -> Result<Self, io::Error> {
    let local_addr = lookup_host(local_addr).await?.next().unwrap();
    let remote_addr = lookup_host(remote_addr).await?.next().unwrap();
    let socket = UdpSocket::bind(local_addr).await?;
    Ok(Self {
      socket,
      remote_addr,
    })
  }
}

#[async_trait]
impl Protocol for GuaranteedUdp {
  async fn send(&mut self, packet: &[u8]) -> Result<usize, io::Error> {
    loop {
      self.socket.send_to(packet, &self.remote_addr).await?;
      let mut ack = [0; 4];
      match timeout(ACK_TIMEOUT, self.socket.recv_from(&mut ack)).await {
        Ok(Ok((len, _src_addr))) => {
          if len == 4 && &ack == b"ACK\n" {
            break;
          }
        }
        Ok(Err(_)) => { /* Socket closed, retry sending the packet */ }
        Err(_) => { /* Timeout occurred, retry sending the packet */ }
      }
    }
    Ok(packet.len())
  }

  async fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, io::Error> {
    let (len, _) = self.socket.recv_from(buffer).await?;
    self.socket.send_to(b"ACK\n", &self.remote_addr).await?;
    Ok(len)
  }

  fn name(&self) -> &str {
    "guaranteed_udp"
  }
}
