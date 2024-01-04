// org.freedesktop.Secret.Collection

use std::collections::HashMap;

use oo7::portal::{Item, Keyring, Secret};
use serde::{Deserialize, Serialize, Serializer};
use zbus::{dbus_interface, zvariant, Error, ObjectServer, SignalContext};
use zvariant::{ObjectPath, OwnedObjectPath, Signature, Type};

use crate::KEYRING;

#[derive(Default, Debug, Deserialize)]
pub struct Collection {
    items: Vec<Item>,
    label: String, // lable == alias ?
    locked: bool,
    created: u64,
    modified: u64,
    path: OwnedObjectPath,
}

#[dbus_interface(name = "org.freedesktop.Secret.Collection")]
impl Collection {
    pub async fn delete(&self, #[zbus(object_server)] object_server: &ObjectServer) -> ObjectPath {
        object_server.remove::<Collection, _>(&self.path).await; // E0283
        ObjectPath::try_from("/").unwrap()
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

    #[dbus_interface(property, name = "Items")]
    async fn items(&self) -> &Vec<Item> {
        &self.items
    }

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

    #[dbus_interface(signal)]
    pub async fn item_created(ctxt: &SignalContext<'_>) -> Result<(), Error>;

    #[dbus_interface(signal)]
    pub async fn item_deleted(ctxt: &SignalContext<'_>) -> Result<(), Error>;

    #[dbus_interface(signal)]
    pub async fn item_changed(ctxt: &SignalContext<'_>) -> Result<(), Error>;
}

impl Serialize for Collection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        OwnedObjectPath::serialize(&self.path, serializer)
    }
}

impl Type for Collection {
    fn signature() -> Signature<'static> {
        ObjectPath::signature()
    }
}

impl Collection {
    pub async fn new(label: String) -> Self {
        Collection {
            items: Vec::new(),
            label: label.clone(),
            locked: false,
            created: 23123, // TODO: add real date here
            modified: 23123,
            path: OwnedObjectPath::try_from(format!(
                "/org/freedesktop/secrets/collection/{}",
                label
            ))
            .unwrap(),
        }
    }
}

pub async fn init_collection_service() -> Collection {
    // wip: eay way to initialize Collection
    let mut collection_service: Collection = Collection::default();

    collection_service
}
