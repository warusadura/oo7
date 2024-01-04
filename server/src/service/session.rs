// org.freedesktop.Secret.Session

use crate::SESSION;
use zbus::{dbus_interface, zvariant};
use zvariant::OwnedObjectPath;

#[derive(Default)]
pub struct Session {
    session_id: String,
}

#[dbus_interface(name = "org.freedesktop.Secret.Service")]
impl Session {
    pub async fn close(&mut self) {
        // WIP
        // SESSION.get().unwrap().remove(&self.session_id);
    }
}

impl Session {
    pub async fn new() -> Self {
        Session::default() // wip
    }
}

pub async fn init_session_service() -> Session {
    // wip
    Session::default()
}
