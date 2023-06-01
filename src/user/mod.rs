use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::Mutex;
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
  networks: Arc<Mutex<Vec<SocketAddr>>>,
}

struct UdpPacket {
  #[allow(dead_code)]
  src_addr: SocketAddr,
  payload: Vec<u8>,
}

impl User {
  pub async fn new(addr: SocketAddr) -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self {
      id: Uuid::new_v4(),
      addr,
      tcp_listener: Arc::new(TcpListener::bind(addr).await.unwrap()),
      udp_listener: Arc::new(UdpSocket::bind(addr).await.unwrap()),
      networks: Arc::new(Mutex::new(Vec::new())),
    }))
  }

  pub async fn send_file<P: Protocol + Send + ?Sized>(
    &self,
    protocol: &mut P,
    file_data: &[u8],
  ) -> Result<usize, io::Error> {
    protocol.send(file_data).await
  }

  pub fn start_listening(self) -> Result<(), io::Error> {
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
    receiver: SocketAddr,
    protocol: ProtocolType,
  ) -> Result<FileTransfer, io::Error> {
    let options = match protocol {
      ProtocolType::Tcp => {
        let protocol = Tcp::new(receiver).await?;
        TransferOptions::new(100, Box::new(protocol))
      }
      ProtocolType::Udp => {
        let protocol = Udp::new(receiver, self.udp_listener.clone()).await?;
        TransferOptions::new(100, Box::new(protocol))
      }
      ProtocolType::GuaranteedUdp => {
        let protocol = GuaranteedUdp::new(self.addr, receiver).await?;
        TransferOptions::new(100, Box::new(protocol))
      }
    };
    let file_transfer = FileTransfer::new(self.clone(), options);

    Ok(file_transfer)
  }

  pub async fn connect_to_network(&self, network_addr: SocketAddr) -> io::Result<()> {
    let mut stream = TcpStream::connect(network_addr).await?;

    // include the protocol in your message
    let message = format!("@enter://{},{}", self.addr, self.id);
    stream.write_all(message.as_bytes()).await?;

    let mut networks = self.networks.lock().await;
    networks.push(network_addr);

    Ok(())
  }

  pub async fn exit_network(&mut self, network_addr: SocketAddr) -> io::Result<()> {
    let mut stream = TcpStream::connect(network_addr).await?;

    let message = format!("@quit://{}", self.id);
    stream.write_all(message.as_bytes()).await?;

    // Remove the network from the user's network list
    let networks = &mut self.networks.lock().await;
    networks.retain(|&addr| addr != network_addr);

    Ok(())
  }

  pub async fn get_address_by_uuid(&self, uuid: Uuid) -> io::Result<SocketAddr> {
    let networks = self.networks.lock().await;
    for network_addr in networks.iter() {
      let mut stream = TcpStream::connect(*network_addr).await?;

      // include the protocol in your message
      let message = format!("@query://{}", uuid);
      stream.write_all(message.as_bytes()).await?;

      let mut buffer = [0; 50];
      let read_bytes = stream.read(&mut buffer[..]).await?;

      let addr: SocketAddr = match String::from_utf8_lossy(&buffer[..read_bytes])
        .trim()
        .parse()
      {
        Ok(addr) => addr,
        Err(_) => continue,
      };

      return Ok(addr);
    }

    Err(io::Error::new(
      io::ErrorKind::NotFound,
      "UUID not found in any network",
    ))
  }
}
