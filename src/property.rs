use std::ops::Deref;

use zvariant::{OwnedSignature, OwnedValue, Signature};

use crate::{
    builder_get,
    error::{BuilderError, PropertyError, ValueError},
    value::{Nullable, Value},
};

#[derive(Debug, serde::Deserialize, serde::Serialize, zvariant::Type)]
pub struct Property {
    name: String,
    about: String,
    long_about: String,
    default: Value,
    current: Value,
    choices: Vec<Value>,
    show_in_settings: bool,
    writable: bool,
    sig: OwnedSignature,
}

impl PartialEq for Property {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Into<Value> for Property {
    fn into(self) -> Value {
        self.current
    }
}

impl Property {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn signature(&self) -> Signature<'static> {
        self.sig.deref().clone()
    }

    /// Resets to default value if there is one provided. Returns if proeprty is not writable
    pub fn reset(&mut self) -> bool {
        if !self.writable {
            return false;
        }
        self.current = self.default.clone();
        return true;
    }

    pub fn get_value(&self) -> Option<&OwnedValue> {
        self.current.get_inner()
    }

    pub fn set_value(&mut self, value: Value) -> anyhow::Result<()> {
        if !self.writable {
            return Err(
                anyhow::Error::new(PropertyError::NotWritable).context(format!(
                    "WritableError occured while trying to write to {}",
                    self.name
                )),
            );
        }

        if value.signature() != *self.sig {
            let err = anyhow::Error::new(ValueError::SignatureNotMatched).context(format!(
                "Condition Failed: sig '{}' == sig '{}'",
                value.signature(),
                self.sig
            ));
            return Err(err);
        }

        if !self.choices.is_empty() && !self.choices.contains(&value) {
            return Err(
                anyhow::Error::new(PropertyError::NotFoundInChoices).context(format!(
                    "The value provided to `Property::set_value` is not within choice bound!"
                )),
            );
        }
        self.current = value;

        return Ok(());
    }

    pub fn about(&self) -> &str {
        &self.about
    }

    pub fn long_about(&self) -> &str {
        &self.long_about
    }

    pub fn is_writable(&self) -> bool {
        self.writable
    }

    pub fn show_in_settings(&self) -> bool {
        self.show_in_settings
    }
}

#[derive(Default)]
pub struct PropertyBuilder {
    show_in_settings: Option<bool>,
    writable: Option<bool>,
    current: Option<Value>,
    about: Option<String>,
    long_about: Option<String>,
    signature: Option<Signature<'static>>,
    default: Option<Value>,
    choices: Vec<Value>,
    name: Option<String>,
}

impl PropertyBuilder {
    pub fn about(mut self, about: String) -> Self {
        self.about = Some(about);
        self
    }

    pub fn default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn choice(mut self, choice: Value) -> Self {
        self.choices.push(choice);
        self
    }

    pub fn show_in_settings(mut self, show_in_settings: bool) -> Self {
        self.show_in_settings = Some(show_in_settings);
        self
    }

    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = Some(writable);
        self
    }

    pub fn value(mut self, value: Value) -> Self {
        self.current = Some(value);
        self
    }

    pub fn long_about(mut self, long_about: String) -> Self {
        self.long_about = Some(long_about);
        self
    }

    pub fn choices(mut self, choices: Vec<Value>) -> Self {
        self.choices.extend(choices);
        self
    }

    pub fn choices_sig<T>(mut self, signature: Signature<'static>, choices: Vec<Option<T>>) -> Self
    where
        T: Into<zvariant::Value<'static>> + zvariant::DynamicType,
    {
        choices.into_iter().fold(self, |_self, choice| {
            _self.choice(
                Value::new(choice, signature.clone()).expect(
                    "Failed to insert choice for property, value's signature doesent match",
                ),
            )
        })
    }

    pub fn signature(mut self, signature: Signature<'static>) -> Self {
        self.signature = Some(signature);
        self
    }
}

impl PropertyBuilder {
    pub fn build(self) -> anyhow::Result<Property> {
        let writable = self.writable.unwrap_or(true);
        let show_in_settings = self.show_in_settings.unwrap_or(true);
        let signature = builder_get!(
            self,
            signature,
            "signature",
            "PropertyBuilder",
            format!("Missing property: signature, use `PropertyBuilder::signature` to set")
        );

        let default = self
            .default
            .unwrap_or(Value::new::<u32>(None, signature.clone())?);
        let property = self.current.unwrap_or(default.clone());

        if default.signature() != signature {
            return Err(
                anyhow::Error::new(PropertyError::InvalidSignature).context(format!(
                    "Set the signature to the proper one! IE: u for int"
                )),
            );
        }

        if property.signature() != signature {
            return Err(
                anyhow::Error::new(PropertyError::InvalidSignature).context(format!(
                    "Set the signature to the proper one! IE: u for int"
                )),
            );
        }

        if self
            .choices
            .iter()
            .find(|c| c.signature() != signature)
            .is_some()
        {
            return Err(anyhow::Error::new(PropertyError::InvalidSignature)
                .context(format!("Incorrect Signature for choice provided!")));
        }

        if !self.choices.is_empty()
            && !self.choices.contains(&default) | !self.choices.contains(&property)
        {
            return Err(anyhow::Error::new(PropertyError::NotFoundInChoices).context(format!("The value of either `default` or `value` was not found in the provided choices")));
        }

        Ok(Property {
            about: self.about.unwrap_or("No summary provided".to_string()),
            sig: signature.into(),
            name: builder_get!(
                self,
                name,
                "name",
                "PropertyBuilder",
                format!("Missing a field: name: Please set it using `PropertyBuilder::name`")
            ),
            long_about: self
                .long_about
                .unwrap_or("No description provided".to_string()),
            default: default,
            current: property,
            choices: self.choices,
            show_in_settings: show_in_settings,
            writable: writable,
        })
    }
}

impl Property {
    pub fn builder() -> PropertyBuilder {
        Default::default()
    }
}
