use std::sync::Arc;

use serde::{Deserialize, Serialize, ser::SerializeTuple};
use zbus::zvariant::{OwnedObjectPath, Type};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::Session;
use crate::{Key, Secret, crypto, dbus::Error, secret::ContentType};

#[derive(Debug, Serialize, Deserialize, Type)]
#[zvariant(signature = "(oayays)")]
/// Same as [`DBusSecret`] without tying the session path to a [`Session`] type.
pub struct DBusSecretInner(
    pub OwnedObjectPath,
    pub Vec<u8>,
    pub Vec<u8>,
    pub ContentType,
);

#[derive(Debug, Type, Zeroize, ZeroizeOnDrop)]
#[zvariant(signature = "(oayays)")]
pub struct DBusSecret<'a> {
    #[zeroize(skip)]
    pub(crate) session: Arc<Session<'a>>,
    pub(crate) parameters: Vec<u8>,
    pub(crate) value: Vec<u8>,
    #[zeroize(skip)]
    pub(crate) content_type: ContentType,
}

impl<'a> DBusSecret<'a> {
    pub(crate) fn new(session: Arc<Session<'a>>, secret: impl Into<Secret>) -> Self {
        let secret = secret.into();
        Self {
            session,
            parameters: vec![],
            value: secret.as_bytes().to_vec(),
            content_type: secret.content_type(),
        }
    }

    pub(crate) fn new_encrypted(
        session: Arc<Session<'a>>,
        secret: impl Into<Secret>,
        aes_key: &Key,
    ) -> Result<Self, crate::dbus::Error> {
        let iv = crypto::generate_iv()?;
        let secret = secret.into();
        Ok(Self {
            session,
            value: crypto::encrypt(secret.as_bytes(), aes_key, &iv)?,
            parameters: iv,
            content_type: secret.content_type(),
        })
    }

    pub(crate) async fn from_inner(
        cnx: &zbus::Connection,
        inner: DBusSecretInner,
    ) -> Result<Self, Error> {
        Ok(Self {
            session: Arc::new(Session::new(cnx, inner.0).await?),
            parameters: inner.1,
            value: inner.2,
            content_type: inner.3,
        })
    }

    pub(crate) fn decrypt(&self, key: Option<&Arc<Key>>) -> Result<Secret, Error> {
        let value = match key {
            Some(key) => &crypto::decrypt(&self.value, key, &self.parameters)?,
            None => &self.value,
        };
        Ok(Secret::with_content_type(self.content_type, value))
    }

    /// Session used to encode the secret
    pub fn session(&self) -> &Session<'_> {
        &self.session
    }

    /// Algorithm dependent parameters for secret value encoding
    pub fn parameters(&self) -> &[u8] {
        &self.parameters
    }

    /// Possibly encoded secret value
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    /// Content type of the secret
    pub fn content_type(&self) -> ContentType {
        self.content_type
    }
}

impl Serialize for DBusSecret<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut tuple_serializer = serializer.serialize_tuple(4)?;
        tuple_serializer.serialize_element(self.session().inner().path())?;
        tuple_serializer.serialize_element(self.parameters())?;
        tuple_serializer.serialize_element(self.value())?;
        tuple_serializer.serialize_element(self.content_type().as_str())?;
        tuple_serializer.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature() {
        assert_eq!(DBusSecret::SIGNATURE, "(oayays)");
    }
}
