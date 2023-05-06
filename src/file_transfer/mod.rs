use std::fs;

use crate::protocol::Protocol;
use crate::user::User;

pub struct TransferOptions {
  packet_size: usize,
  pub(crate) protocol: Box<dyn Protocol + Send>,
}

impl TransferOptions {
  pub fn new(packet_size: usize, protocol: Box<dyn Protocol + Send>) -> Self {
    Self {
      packet_size,
      protocol,
    }
  }
}

pub struct FileTransfer {
  sender: User,
  options: TransferOptions,
  progress: TransferProgress,
}

pub enum TransferProgress {
  NotStarted,
  InProgress {
    bytes_sent: usize,
    total_bytes: usize,
  },
  Complete,
  Error(String),
}

impl FileTransfer {
  pub fn new(sender: User, options: TransferOptions) -> Self {
    Self {
      sender,
      options,
      progress: TransferProgress::NotStarted,
    }
  }

  pub async fn send(&mut self, file_path: &str) -> Result<(), String> {
    let file_data = fs::read(file_path).map_err(|e| e.to_string())?;

    let total_bytes = file_data.len();
    self.progress = TransferProgress::InProgress {
      bytes_sent: 0,
      total_bytes,
    };

    let mut bytes_sent = 0;
    let protocol = self.options.protocol.as_mut();
    while bytes_sent < total_bytes {
      let chunk_end = usize::min(bytes_sent + self.options.packet_size, total_bytes);
      let packet = &file_data[bytes_sent..chunk_end];
      match self.sender.send_file(&mut *protocol, packet).await {
        Ok(bytes) => {
          bytes_sent += bytes;
          log::debug!("Sent {} bytes through {}", bytes, protocol.name());
          self.progress = TransferProgress::InProgress {
            bytes_sent,
            total_bytes,
          };
        }
        Err(e) => {
          self.progress = TransferProgress::Error(e.to_string());
          return Err(e.to_string());
        }
      }
    }

    self.progress = TransferProgress::Complete;
    Ok(())
  }

  pub fn progress(&self) -> &TransferProgress {
    &self.progress
  }
}
