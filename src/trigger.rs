use std::ops::Deref;

use zvariant::{DynamicType, OwnedSignature, OwnedValue, Signature, Type};

use crate::value::Value;

#[derive(serde::Serialize, Debug, serde::Deserialize, zvariant::Type, zvariant::Value)]
pub struct Trigger {
    name: String,
    signature: OwnedSignature,
}

impl PartialEq for Trigger {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Trigger {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn new(name: String, signature: Signature<'static>) -> Self {
        Self {
            name: name,
            signature: signature.into(),
        }
    }
    
    pub fn signature(&self) -> Signature<'static> {
        self.signature.deref().clone()
    }
    pub fn matches(&self, value: &OwnedValue) -> bool {
        value.value_signature() == self.signature.as_str()
    }
}
