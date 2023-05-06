use async_trait::async_trait;

#[async_trait]
pub trait Protocol {
  async fn send(&mut self, packet: &[u8]) -> Result<usize, std::io::Error>;
  async fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, std::io::Error>;
  fn name(&self) -> &str;
}

pub enum ProtocolType {
  Tcp,
  Udp,
  GuaranteedUdp,
}

pub mod guaranteed_udp;
pub mod tcp;
pub mod udp;
