// org.freedesktop.Secret.Collection

use std::collections::HashMap;

use oo7::portal::{Item, Keyring, Secret};
use serde::{Deserialize, Serialize};
use zbus::{dbus_interface, zvariant, Error, ObjectServer, SignalContext};
use zvariant::{OwnedObjectPath, Type};

use crate::KEYRING;

#[derive(Default, Debug, Deserialize, Serialize, Type)]
pub struct Collection {
    items: Vec<Item>,
    label: String,
    alias: String,
    locked: bool,
    created: u64,
    modified: u64,
}

#[dbus_interface(name = "org.freedesktop.Secret.Collection")]
impl Collection {
    pub async fn delete(
        &self,
        prompt: OwnedObjectPath,
        #[zbus(object_server)] object_server: &ObjectServer,
    ) {
        // object_server.remove(prompt).await; // E0282
    }

    pub async fn search_items(&self, attributes: HashMap<&str, &str>) -> Vec<Item> {
        let items = match KEYRING.get().unwrap().search_items(attributes).await {
            Ok(i) => i,
            Err(_) => todo!(),
        };

        items
    }

    pub async fn create_item(&self, properties: HashMap<&str, &str>, secret: &str, replace: bool) {
        // specification method signature missing 'label' parameter
        // TODO: return
        let _ = KEYRING
            .get()
            .unwrap()
            .create_item("label", properties, secret, replace)
            .await;
    }

    /*
    #[dbus_interface(property, name = "Items")]
    async fn items(&self) -> &Vec<Item> {
        &self.items
    }*/

    #[dbus_interface(property, name = "Label")]
    pub async fn label(&self) -> &String {
        &self.label
    }

    #[dbus_interface(property, name = "Locked")]
    pub async fn locked(&self) -> bool {
        self.locked
    }

    #[dbus_interface(property, name = "Created")]
    pub async fn created(&self) -> u64 {
        self.created
    }

    #[dbus_interface(property, name = "Modified")]
    pub async fn modified(&self) -> u64 {
        self.modified
    }

    pub async fn alias(&self) -> &String {
        &self.alias
    }

    #[dbus_interface(signal)]
    pub async fn item_created(ctxt: &SignalContext<'_>) -> Result<(), Error>;

    #[dbus_interface(signal)]
    pub async fn item_deleted(ctxt: &SignalContext<'_>) -> Result<(), Error>;

    #[dbus_interface(signal)]
    pub async fn item_changed(ctxt: &SignalContext<'_>) -> Result<(), Error>;
}

impl Collection {
    pub async fn new(label: String) -> Self {
        Collection {
            items: Vec::new(),
            label: label,
            alias: String::from("new"),
            locked: false,
            created: 23123, // TODO add real date
            modified: 23123,
        }
    }
}

/*
async fn lookup_existing_keyrings() -> Vec<Keyring> {
    check .local/share/keyrings/ dir
    load keyrings with portal::Keyring::load
    insert discovered keyring to collections vec
}
*/

pub async fn init_collection_service() -> Collection {
    let mut collection_service: Collection = Collection::default(); // wip

    collection_service
}
