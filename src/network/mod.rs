use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::user::User;

pub struct Network {
  users: Arc<Mutex<HashMap<Uuid, User>>>,
}

impl Network {
  pub fn new() -> Self {
    Self {
      users: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn add_user(&self, user: User) {
    let mut users = self.users.lock().unwrap();
    users.insert(*user.id(), user);
  }

  pub fn remove_user(&self, user_id: &Uuid) {
    let mut users = self.users.lock().unwrap();
    users.remove(user_id);
  }

  pub fn get_user(&self, user_id: &Uuid) -> Option<User> {
    let users = self.users.lock().unwrap();
    users.get(user_id).cloned()
  }
}

impl Default for Network {
  fn default() -> Self {
    Self::new()
  }
}
