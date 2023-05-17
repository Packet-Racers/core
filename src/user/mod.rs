use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};
use uuid::Uuid;

use crate::file_transfer::{FileTransfer, TransferOptions};
use crate::protocol::guaranteed_udp::GuaranteedUdp;
use crate::protocol::tcp::Tcp;
use crate::protocol::udp::Udp;
use crate::protocol::{Protocol, ProtocolType};

#[derive(Clone)]
pub struct User {
  id: Uuid,
  addr: SocketAddr,
  tcp_listener: Arc<TcpListener>,
  udp_listener: Arc<UdpSocket>,
}

struct UdpPacket {
  src_addr: SocketAddr,
  payload: Vec<u8>,
}

impl User {
  pub async fn new(addr: SocketAddr) -> Self {
    Self {
      id: Uuid::new_v4(),
      addr,
      tcp_listener: Arc::new(TcpListener::bind(addr).await.unwrap()),
      udp_listener: Arc::new(UdpSocket::bind(addr).await.unwrap()),
    }
  }

  pub async fn send_file<P: Protocol + Send + ?Sized>(
    &self,
    protocol: &mut P,
    file_data: &[u8],
  ) -> Result<usize, io::Error> {
    protocol.send(file_data).await
  }

  pub fn start_listening(self: Arc<Self>) -> Result<(), io::Error> {
    let tcp_listener = Arc::clone(&self.tcp_listener);
    let udp_listener = Arc::clone(&self.udp_listener);

    tokio::spawn(async move {
      let mut buffer = [0; 1024];
      loop {
        let (mut stream, src_addr) = tcp_listener.accept().await.unwrap();
        log::debug!("[{}] Received connection from: {}", self.addr, src_addr);

        let mut file = tokio::fs::File::create("received.txt").await.unwrap();
        loop {
          let number_of_bytes = stream.read(&mut buffer).await.unwrap();
          log::debug!(
            "[{}] Received {} bytes from: {}",
            self.addr,
            number_of_bytes,
            src_addr
          );

          if number_of_bytes == 0 {
            break;
          }

          file.write_all(&buffer[..number_of_bytes]).await.unwrap();
        }
      }
    });

    tokio::spawn(async move {
      let mut buffer = [0; 1024];
      println!("Waiting for UDP connection...");

      loop {
        let (number_of_bytes, src_addr) = udp_listener.recv_from(&mut buffer).await.unwrap();
        println!("Received {} bytes from: {}", number_of_bytes, src_addr);

        let packet = UdpPacket {
          src_addr,
          payload: buffer[..number_of_bytes].to_vec(),
        };

        Self::handle_udp_packet(packet).await;
      }
    });

    Ok(())
  }

  async fn handle_udp_packet(packet: UdpPacket) {
    let UdpPacket { payload, .. } = packet;

    let mut file = tokio::fs::OpenOptions::new()
      .create(true)
      .append(true)
      .open("udp.txt")
      .await
      .unwrap();

    file.write_all(&payload).await.unwrap();
  }

  pub fn id(&self) -> &Uuid {
    &self.id
  }

  pub fn addr(&self) -> &SocketAddr {
    &self.addr
  }

  pub async fn create_file_transfer(
    &self,
    receiver: Arc<Self>,
    protocol: ProtocolType,
  ) -> Result<FileTransfer, io::Error> {
    let options = match protocol {
      ProtocolType::Tcp => {
        let protocol = Tcp::new(receiver.addr()).await?;
        TransferOptions::new(100, Box::new(protocol))
      }
      ProtocolType::Udp => {
        let protocol = Udp::new(receiver.addr().to_owned(), self.udp_listener.clone()).await?;
        TransferOptions::new(100, Box::new(protocol))
      }
      ProtocolType::GuaranteedUdp => {
        let protocol = GuaranteedUdp::new(self.addr, receiver.addr().to_owned()).await?;
        TransferOptions::new(100, Box::new(protocol))
      }
    };
    let file_transfer = FileTransfer::new(self.clone(), options);

    Ok(file_transfer)
  }
}
