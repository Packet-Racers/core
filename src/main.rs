use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use packet_racers::logging::init_library_logger;
use packet_racers::protocol::ProtocolType;
use packet_racers::user::User;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  init_library_logger(log::LevelFilter::Debug);

  // Create two arc users
  let alice = Arc::new(User::new(SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    8000,
  )));
  let bob = Arc::new(User::new(SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    8001,
  )));

  // Start both user listening for incoming connections
  if let Err(e) = alice.clone().start_listening() {
    println!("Error listening for alice: {}", e);
  }

  if let Err(e) = bob.clone().start_listening() {
    println!("Error listening for bob: {}", e);
  }

  alice
    .create_file_transfer(bob.clone(), ProtocolType::Tcp)
    .await?
    .send("Cargo.toml")
    .await?;

  Ok(())
}
