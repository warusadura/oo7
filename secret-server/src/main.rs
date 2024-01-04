pub mod service;

use std::collections::HashMap;

use once_cell::sync::OnceCell;
use oo7::portal::Keyring;
use std::future::pending;
use zbus::{ConnectionBuilder, Result};

use crate::service::collection::init_collection_service;
use crate::service::service::init_secret_service;
use crate::service::session::{init_session_service, Session};

pub static KEYRING: OnceCell<Keyring> = OnceCell::new();
pub static SESSION: OnceCell<HashMap<String, Session>> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<()> {
    let secret_service = init_secret_service().await;
    let collection = init_collection_service().await;
    let session = init_session_service().await;

    let _service = ConnectionBuilder::session()?
        .name("org.freedesktop.secrets.new")?
        .serve_at("/org/freedesktop/secrets/new", secret_service)?
        .serve_at("/org/freedesktop/secrets/new/collection", collection)?
        .serve_at("/org/freedesktop/secrets/new/session", session)?
        .build()
        .await?;

    let keyring = match Keyring::load_default().await {
        Ok(i) => KEYRING.set(i),
        Err(_) => todo!(), // call some init to create default keyring
    };

    pending::<()>().await;

    Ok(())
}
