use async_trait::async_trait;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};

use super::Protocol;

pub struct Tcp {
  stream: TcpStream,
}

impl Tcp {
  pub async fn new<A: ToSocketAddrs>(addr: A) -> Result<Self, io::Error> {
    let stream = TcpStream::connect(addr).await?;
    stream.set_nodelay(true)?;
    Ok(Self { stream })
  }
}

#[async_trait]
impl Protocol for Tcp {
  async fn send(&mut self, packet: &[u8]) -> Result<usize, io::Error> {
    self.stream.write_all(packet).await?;
    Ok(packet.len())
  }

  async fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, io::Error> {
    self.stream.read(buffer).await
  }

  fn name(&self) -> &str {
    "tcp"
  }
}
