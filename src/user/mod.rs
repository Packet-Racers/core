use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use std::{io, net::TcpListener};
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
}

impl User {
  pub fn new(addr: SocketAddr) -> Self {
    Self {
      id: Uuid::new_v4(),
      addr,
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
    // Listen for both TCP and UDP in the same address in different threads
    let tcp_listener = TcpListener::bind(self.addr)?;
    // let udp_listener = self.addr;

    // Save the file received in the TCP lister as tcp.txt
    thread::spawn(move || {
      let mut buffer = [0; 1024];
      let (mut stream, src_addr) = tcp_listener.accept().unwrap();
      log::debug!("[{}] Received connection from: {}", self.addr, src_addr);

      let mut file = std::fs::File::create("received.txt").unwrap();
      loop {
        let number_of_bytes = stream.read(&mut buffer).unwrap();
        log::debug!(
          "[{}] Received {} bytes from: {}",
          self.addr,
          number_of_bytes,
          src_addr
        );

        if number_of_bytes == 0 {
          break;
        }

        file.write_all(&buffer[..number_of_bytes]).unwrap();
      }
    });

    // Save the file received in the UDP lister as udp.txt
    // thread::spawn(move || {
    //   let mut buffer = [0; 1024];
    //   let socket = std::net::UdpSocket::bind(udp_listener).unwrap();
    //
    //   loop {
    //     let (number_of_bytes, src_addr) = socket.recv_from(&mut buffer).unwrap();
    //     println!("Received {} bytes from: {}", number_of_bytes, src_addr);
    //
    //     let mut file = std::fs::File::create("udp.txt").unwrap();
    //     file.write_all(&buffer).unwrap();
    //   }
    // });

    Ok(())
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
        let protocol = Udp::new(receiver.addr().to_owned()).await?;
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
