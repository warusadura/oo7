//  org.freedesOnceCellktop.Secret.Service

use std::collections::HashMap;

use oo7::portal::{Item, Keyring, Secret};
use serde::{Deserialize, Serialize};
use zbus::{dbus_interface, zvariant, Connection, Error, ObjectServer, SignalContext};
use zvariant::{OwnedObjectPath, Type};

use crate::service::collection::Collection;
use crate::service::session::Session;
use crate::KEYRING;

#[derive(Deserialize, Serialize, Type, Debug)]
pub struct Service {
    collections: Vec<Collection>,
}

#[dbus_interface(name = "org.freedesktop.Secret.Service")]
impl Service {
    pub async fn open_session(&self, algorithm: &str, input: &str) {
        // WIP
        let session = Session::new();
    }

    pub async fn create_collection(
        &self,
        #[zbus(object_server)] object_server: &ObjectServer,
        properties: HashMap<&str, &str>,
        alias: String,
    ) {
        // WIP
        let collection = Collection::new(alias);
        // object_server.at(format!("/org/freedesktop/secrets/new/collection/{}", alias), collection);
        // zbus::Interface` is not satisfied
    }

    pub async fn search_items(&self, attributes: HashMap<&str, &str>) {
        let items = match KEYRING.get().unwrap().search_items(attributes).await {
            Ok(i) => i,
            Err(_) => todo!(),
        };

        let mut unlocked: Vec<Item>;
        let mut locked: Vec<Item>;

        for item in items {
            // TODO
            // needs a way to access item's locked or unlocked status
        }
    }

    pub async fn unlock(&self, objects: Vec<OwnedObjectPath>) {
        // TODO
        /*
         * loop through provided objects accessing locked state
         * if it's already locked unlock
         * otherwise do nothing
         */
    }

    pub async fn lock(&self, objects: Vec<OwnedObjectPath>) {
        // TODO
        // opposite impl of unlock
    }

    pub async fn get_secrets(&self, items: Vec<OwnedObjectPath>) {
        // TODO
    }

    pub async fn read_alias(&self, name: &str) {
        // TODO return
        for collection in &self.collections {
            if collection.label().await == name {
                // return collection objectpath
            } else {
                // return "/"
            }
        }
    }

    pub async fn set_alias(&self, name: &str, collection: &str) {
        // TODO
    }

    /*
    #[dbus_interface(property)]
     pub async fn get_collections(&self) -> &Vec<Collection> {
         &self.collections
     }*/

    #[dbus_interface(signal)]
    pub async fn collection_created(ctxt: &SignalContext<'_>) -> Result<(), Error>;

    #[dbus_interface(signal)]
    pub async fn collection_deleted(ctxt: &SignalContext<'_>) -> Result<(), Error>;

    #[dbus_interface(signal)]
    pub async fn collection_changed(ctxt: &SignalContext<'_>) -> Result<(), Error>;
}

pub async fn init_secret_service() -> Service {
    Service {
        collections: Vec::new(),
    }
}

/*
async fn lookup_existing_keyrings() -> Vec<Keyring> {
    check .local/share/keyrings/ dir
    load keyrings with portal::Keyring::load
    insert discovered keyring to collections vec
}
*/
