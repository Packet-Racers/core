use async_trait::async_trait;

#[async_trait]
pub trait Protocol {
  async fn send(&mut self, packet: &[u8]) -> Result<usize, std::io::Error>;
  async fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, std::io::Error>;
  fn name(&self) -> &str;
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ProtocolType {
  Tcp,
  Udp,
  GuaranteedUdp,
}

impl std::str::FromStr for ProtocolType {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "tcp" | "Tcp" => Ok(Self::Tcp),
      "udp" | "Udp" => Ok(Self::Udp),
      "gudp" | "GuaranteedUdp" => Ok(Self::GuaranteedUdp),
      _ => Err(format!("Unknown protocol: {}", s)),
    }
  }
}

// #[cfg(feature = "guaranteed_udp")]
pub mod guaranteed_udp;

// #[cfg(feature = "tcp")]
pub mod tcp;

// #[cfg(feature = "udp")]
pub mod udp;
