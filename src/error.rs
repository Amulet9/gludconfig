#[derive(thiserror::Error, Debug)]
pub enum ValueError {
    #[error("The value's signature, and Signature passed to `Value::new` dont match")]
    SignatureNotMatched,
}

#[derive(thiserror::Error, Debug)]
pub enum ZbusError<'a> {
    #[error("Schema with name {0} not found")]
    SchemaNotFound(&'a str),
    #[error("Property not found in schema {0} with name {1}")]
    PropertyNotFound(&'a str, &'a str),
    #[error("Trigger not found in schema {0} with name {1}")]
    TriggerNotFound(&'a str, &'a str),
}


#[cfg(feature = "dbus")]
impl<'a> Into<zbus::fdo::Error> for ZbusError<'a> {
    fn into(self) -> zbus::fdo::Error {
        zbus::fdo::Error::Failed(format!("{}", self))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("No home directory found for user {0}")]
    NoHomeFound(&'static str),
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {}

#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("The value {0} was not set in builder {1}")]
    UnwrapFailed(&'static str, &'static str),
}
#[derive(Debug, thiserror::Error)]
pub enum PropertyError {
    #[error("The provided values were not inside acceptable choices.")]
    NotFoundInChoices,
    #[error("The signature of `default` or `value` do not match the one passed by the user")]
    InvalidSignature,
    #[error("Property cannot be writed to, as its `writable` field is set to false")]
    NotWritable,
}

impl BuilderError {
    pub fn unwrap_failed(value: &'static str, builder: &'static str, ctx: String) -> anyhow::Error {
        anyhow::Error::new(Self::UnwrapFailed(value, builder)).context(ctx)
    }
}
