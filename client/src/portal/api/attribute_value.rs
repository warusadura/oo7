use serde::{Deserialize, Serialize};
use zbus::zvariant::{Type, Value};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::{crypto, Key};

/// An encrypted attribute value.
#[derive(Deserialize, Serialize, Type, Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct AttributeValue(String);

impl AttributeValue {
    pub(crate) fn mac(&self, key: &Key) -> Zeroizing<Vec<u8>> {
        Zeroizing::new(crypto::compute_mac(self.0.as_bytes(), key))
    }
}

impl Into<Value<'_>> for AttributeValue {
	fn into(self) -> Value<'static> {
		Value::Str(self.0.clone().into())
	}
}

impl TryFrom<Value<'_>> for AttributeValue {
	type Error = zbus::zvariant::Error;
	fn try_from(v: Value<'_>) -> Result<AttributeValue, Self::Error> {
		Ok(AttributeValue(v.try_into()?))
	}
}

impl From<&str> for AttributeValue {
	fn from(value: &str) -> Self {
		Self(value.to_string())
	}
}

impl From<String> for AttributeValue {
	fn from(value: String) -> Self {
		Self(value)
	}
}

//impl<S: ToString> From<S> for AttributeValue {
//    fn from(value: S) -> Self {
//        Self(value.to_string())
//    }
//}

impl AsRef<str> for AttributeValue {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl std::ops::Deref for AttributeValue {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}
