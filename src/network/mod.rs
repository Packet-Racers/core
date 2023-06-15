use std::collections::HashMap;

use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

pub struct Network {
  connections: HashMap<Uuid, SocketAddr>,
  listener: TcpListener,
}

impl Network {
  pub async fn new(port: u16) -> std::io::Result<Self> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    Ok(Self {
      connections: HashMap::new(),
      listener,
    })
  }

  pub async fn listen(&mut self) -> std::io::Result<()> {
    loop {
      match self.listener.accept().await {
        Ok((mut stream, _)) => {
          let mut buffer = [0; 1028];
          let read_bytes = stream.read(&mut buffer[..]).await?;

          let data = String::from_utf8_lossy(&buffer[..read_bytes]);
          let parts: Vec<&str> = data.split("://").collect();

          let protocol = parts[0];
          let data = parts[1];

          self.handle_protocol(protocol, data, &mut stream).await?;
        }
        Err(e) => {
          eprintln!("Connection failed because: {}", e);
        }
      }
    }
  }

  async fn handle_protocol(
    &mut self,
    protocol: &str,
    data: &str,
    stream: &mut TcpStream,
  ) -> std::io::Result<()> {
    match protocol {
      "@enter" => self.handle_enter(data).await,
      "@query" => self.handle_query(data, stream).await,
      "@quit" => self.handle_quit(data).await,
      _ => Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "Invalid protocol",
      )),
    }
  }

  async fn handle_enter(&mut self, data: &str) -> std::io::Result<()> {
    let parts: Vec<&str> = data.split(',').collect();
    let addr: SocketAddr = parts[0].parse().unwrap();
    let uuid: Uuid = parts[1].parse().unwrap();

    println!("{} entered the network", uuid);

    self.connections.insert(uuid, addr);

    Ok(())
  }

  async fn handle_query(&self, data: &str, stream: &mut TcpStream) -> std::io::Result<()> {
    println!("Querying for {}", data);

    let uuid: Uuid = data.parse().unwrap();

    println!("{} is at {:?}", uuid, self.get_address(uuid));

    match self.get_address(uuid) {
      Some(addr) => {
        let addr_str = addr.to_string();
        stream.write_all(addr_str.as_bytes()).await?;
      }
      None => {
        let response = "UUID not found";
        stream.write_all(response.as_bytes()).await?;
      }
    }
    Ok(())
  }

  async fn handle_quit(&mut self, data: &str) -> std::io::Result<()> {
    let uuid: Uuid = data.parse().unwrap();
    self.connections.remove(&uuid);
    println!("{} left the network", uuid);
    Ok(())
  }

  pub fn get_address(&self, uuid: Uuid) -> Option<&SocketAddr> {
    self.connections.get(&uuid)
  }
}
