#![feature(async_closure)]

#[cfg(feature = "logging")]
pub mod logging;

#[cfg(feature = "network")]
pub mod network;

// If any of the features protocol-udp, protocol-tcp, or protocol-guaranteed-udp are enabled, then
// the protocol module is enabled.
#[cfg(any(
  feature = "protocol-udp",
  feature = "protocol-tcp",
  feature = "protocol-guaranteed-udp"
))]
pub mod file_transfer;
#[cfg(any(
  feature = "protocol-udp",
  feature = "protocol-tcp",
  feature = "protocol-guaranteed-udp"
))]
pub mod protocol;
#[cfg(any(
  feature = "protocol-udp",
  feature = "protocol-tcp",
  feature = "protocol-guaranteed-udp"
))]
pub mod user;
