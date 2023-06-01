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

#[cfg(feature = "guaranteed_udp")]
pub mod guaranteed_udp;

#[cfg(feature = "tcp")]
pub mod tcp;

#[cfg(feature = "udp")]
pub mod udp;
