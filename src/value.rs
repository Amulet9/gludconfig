use std::ops::{Deref, DerefMut};

use zvariant::{to_bytes, OwnedSignature, OwnedValue, Signature};

use crate::error::ValueError;

#[derive(
    serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, zvariant::Type, zvariant::Value,
)]
pub struct Nullable {
    is_null: bool,
    value: OwnedValue,
}

impl Default for Nullable {
    fn default() -> Self {
        Self {
            is_null: true,
            value: zvariant::Value::from("this value is null").into(),
        }
    }
}

impl Into<Option<OwnedValue>> for Nullable {
    fn into(self) -> Option<OwnedValue> {
        self.is_null.then(|| None).unwrap_or(Some(self.value))
    }
}

impl From<Option<OwnedValue>> for Nullable {
    fn from(value: Option<OwnedValue>) -> Self {
        value
            .map(|value| Self {
                is_null: false,
                value,
            })
            .unwrap_or(Self::default())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, zvariant::Type)]
pub struct Value {
    signature: zvariant::OwnedSignature,
    value: Nullable,
}

impl Value {
    pub fn wrap<T>(value: Option<T>) -> Self
    where
        T: Into<zvariant::Value<'static>> + zvariant::Type + zvariant::DynamicType,
    {
        Self::new(value, <T as zvariant::Type>::signature()).unwrap()
    }

    fn check_sig(value: &OwnedValue, sig: &Signature<'static>) -> anyhow::Result<()> {
        (&value.value_signature() != sig)
            .then(|| {
                let err = anyhow::Error::new(ValueError::SignatureNotMatched).context(format!(
                    "Condition Failed: sig '{}' == sig '{}'",
                    value.value_signature(),
                    sig
                ));
                Err(err)
            })
            .unwrap_or(Ok(()))
    }

    pub fn new<T>(value: Option<T>, sig: Signature<'static>) -> anyhow::Result<Self>
    where
        T: Into<zvariant::Value<'static>> + zvariant::DynamicType,
    {
        value
            .map(|value| {
                let value = zvariant::OwnedValue::from(value.into());

                Value::check_sig(&value, &sig)?;

                Ok(Self {
                    signature: OwnedSignature::from(sig.clone()),
                    value: Some(value).into(),
                })
            })
            .unwrap_or(Ok(Self {
                signature: OwnedSignature::from(sig),
                value: None.into(),
            }))
    }

    pub fn signature(&self) -> Signature<'static> {
        self.signature.deref().clone()
    }

    pub fn get_inner(&self) -> Option<&OwnedValue> {
        self.value
            .is_null
            .then(|| None)
            .unwrap_or(Some(&self.value.value))
    }

    pub fn set_value<T>(&mut self, value: Option<T>) -> anyhow::Result<Option<OwnedValue>>
    where
        T: Into<zvariant::Value<'static>> + zvariant::DynamicType,
    {
        value
            .map(|value| {
                let owned_value = zvariant::OwnedValue::from(value.into());
                Value::check_sig(&owned_value, &self.signature)?;
                Ok(std::mem::replace(&mut self.value, Some(owned_value).into()).into())
            })
            .unwrap_or(Ok(std::mem::take(&mut self.value).into()))
    }
}

impl Into<Nullable> for Value {
    fn into(self) -> Nullable {
        self.value
    }
}
