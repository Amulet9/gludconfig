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
        if self.is_null {
            return None;
        } else {
            return Some(self.value);
        }
    }
}

impl From<Option<OwnedValue>> for Nullable {
    fn from(value: Option<OwnedValue>) -> Self {
        match value {
            Some(value) => Self {
                is_null: false,
                value,
            },
            None => Self::default(),
        }
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
        match value {
            Some(value) => {
                let owned_value = OwnedValue::from(value.into());
                let owned_sig = OwnedSignature::from(owned_value.value_signature());

                return Self {
                    signature: owned_sig,
                    value: Some(owned_value).into(),
                };
            }
            None => {
                return Self {
                    signature: OwnedSignature::from(<T as zvariant::Type>::signature()),
                    value: None.into(),
                }
            }
        }
    }

    pub fn check_sig(value: &OwnedValue, sig: &Signature<'static>) -> anyhow::Result<()> {
        if &value.value_signature() != sig {
            let err = anyhow::Error::new(ValueError::SignatureNotMatched).context(format!(
                "Condition Failed: sig '{}' == sig '{}'",
                value.value_signature(),
                sig
            ));
            return Err(err);
        } else {
            return Ok(());
        }
    }

    /// Creates a new value with the provided signatures
    /// Runtime checks for type safety
    pub fn new<T>(value: Option<T>, sig: Signature<'static>) -> anyhow::Result<Self>
    where
        T: Into<zvariant::Value<'static>> + zvariant::DynamicType,
    {
        match value {
            Some(value) => {
                let value = zvariant::OwnedValue::from(value.into());

                Value::check_sig(&value, &sig)?;

                return Ok(Self {
                    signature: OwnedSignature::from(sig),
                    value: Some(value).into(),
                });
            }
            None => {
                return Ok(Self {
                    signature: OwnedSignature::from(sig),
                    value: None.into(),
                })
            }
        }
    }
    /// The signature of the inner type, this can be used for type safety
    pub fn signature(&self) -> Signature<'static> {
        self.signature.deref().clone()
    }

    pub fn get_inner(&self) -> Option<&OwnedValue> {
        match self.value.is_null {
            true => None,
            false => Some(&self.value.value),
        }
    }

    pub fn into_nullabe(self) -> Nullable {
        self.value
    }

    /// Replaces the current value with the provided one, returning the old one.
    /// Runtime checks for type safety
    pub fn set_value<T>(&mut self, value: Option<T>) -> anyhow::Result<Option<OwnedValue>>
    where
        T: Into<zvariant::Value<'static>> + zvariant::DynamicType,
    {
        match value {
            Some(new_value) => {
                let new_value = zvariant::OwnedValue::from(new_value.into());

                Value::check_sig(&new_value, &self.signature)?;
                let old_value = std::mem::take(&mut self.value);
                self.value = Some(new_value).into();

                return Ok(old_value.into());
            }
            None => Ok(std::mem::take(&mut self.value).into()),
        }
    }
}
