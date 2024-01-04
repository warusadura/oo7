// org.freedesktop.Secret.Prompt

use zbus::{dbus_interface, zvariant, Error, SignalContext};
use zvariant::OwnedObjectPath;

#[derive(Default, Debug)]
pub struct Prompt;

#[dbus_interface(name = "org.freedesktop.Secret.Prompt")]
impl Prompt {
    pub async fn prompt(&self) {
        // TODO
    }

    pub async fn dismiss(&self) {
        // TODO
    }

    #[dbus_interface(signal)]
    pub async fn completed(ctxt: &SignalContext<'_>) -> Result<(), Error>;
}
