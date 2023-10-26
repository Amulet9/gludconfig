#[cfg(feature = "dbus")]
mod interface {
    use std::{collections::BTreeMap, sync::Arc};

    use gludconfig::{
        error::ZbusError, property::Property, schema::Schema, storage::Storage, trigger::Trigger,
        value::Nullable,
    };

    use zbus::{dbus_interface, SignalContext};
    use zvariant::{dbus, from_slice, OwnedSignature, OwnedValue, Signature};

    pub struct PropertyInterface {
        pub storage: Arc<Storage>,
    }

    pub struct TriggerInterface {
        pub storage: Arc<Storage>,
    }

    pub struct SchemaInterface {
        pub storage: Arc<Storage>,
    }

    #[dbus_interface(name = "org.glud.GludConfig.Schema")]
    impl SchemaInterface {
        #[dbus_interface(name = "all")]
        async fn all(&self) -> zbus::fdo::Result<Vec<SchemaInfo>> {
            Ok(self
                .storage
                .fetch_all()
                .await?
                .into_iter()
                .map(<Schema as Into<SchemaInfo>>::into)
                .collect())
        }

        #[dbus_interface(name = "metadata")]
        async fn metadata(&self, schema_name: String) -> zbus::fdo::Result<SchemaInfo> {
            Ok(self.storage.get_schema(schema_name).await?.into())
        }
        #[dbus_interface(name = "reset_all")]
        async fn reset_all(
            &self,
            #[zbus(signal_context)] ctx: SignalContext<'_>,
            schema_name: String,
        ) -> zbus::fdo::Result<bool> {
            let mut schema = self.storage.get_schema(schema_name.clone()).await?;
            let mut res = true;
            for p in schema.properties_mut() {
                if Property::reset(p) {
                    ctx.connection()
                        .emit_signal(
                            Some("org.glud.GludConfig"),
                            "/org/glud/gludconfig/property",
                            "org.glud.GludConfig.Property",
                            "property_changed",
                            &(schema_name.clone(), p.name().clone()),
                        )
                        .await?;
                } else {
                    res = false;
                }
            }
            Ok(res)
        }
    }

    #[dbus_interface(name = "org.glud.GludConfig.Trigger")]
    impl TriggerInterface {
        #[dbus_interface(name = "metadata")]
        async fn metadata(
            &self,
            schema_name: String,
            trigger_name: String,
        ) -> zbus::fdo::Result<TriggerInfo> {
            let mut schema = self.storage.get_schema(schema_name.clone()).await?;
            let trigger = schema
                .into_triggers()
                .find(|p| p.name() == &trigger_name)
                .ok_or(Into::<zbus::fdo::Error>::into(ZbusError::TriggerNotFound(
                    &schema_name,
                    &trigger_name,
                )))?;
            return Ok(trigger.into());
        }

        #[dbus_interface(name = "invoke_trigger")]
        async fn trigger(
            &self,
            #[zbus(signal_context)] ctx: SignalContext<'_>,
            schema_name: String,
            trigger_name: String,
            value: OwnedValue,
        ) -> zbus::fdo::Result<()> {
            let mut schema = self.storage.get_schema(schema_name.clone()).await?;
            let trigger = schema
                .triggers()
                .find(|p| p.name() == &trigger_name)
                .ok_or(Into::<zbus::fdo::Error>::into(ZbusError::TriggerNotFound(
                    &schema_name,
                    &trigger_name,
                )))?;
            if trigger.matches(&value) {
                Self::trigger_invoked(&ctx, schema_name, value).await?;
                return Ok(());
            } else {
                return Err(zbus::fdo::Error::Failed(format!(
                    "The signature of the trigger and the provided value dont match"
                )));
            }
        }

        #[dbus_interface(signal, name = "trigger_invoked")]
        async fn trigger_invoked(
            ctx: &SignalContext<'_>,
            signal: String,
            value: OwnedValue,
        ) -> zbus::Result<()>;
    }

    #[dbus_interface(name = "org.glud.GludConfig.Property")]
    impl PropertyInterface {
        #[dbus_interface(signal, name = "property_changed")]
        async fn property_changed(
            ctx: &SignalContext<'_>,
            schema_name: String,
            key_name: String,
        ) -> zbus::Result<()>;

        #[dbus_interface(name = "set")]
        async fn set(
            &self,
            #[zbus(signal_context)] signal_ctx: SignalContext<'_>,
            schema_name: String,
            key_name: String,
            set_value: Nullable,
        ) -> zbus::fdo::Result<()> {
            let mut schema = self.storage.get_schema(schema_name.clone()).await?;
            let property = schema
                .properties_mut()
                .find(|p| p.name() == &key_name)
                .ok_or(Into::<zbus::fdo::Error>::into(ZbusError::PropertyNotFound(
                    &schema_name,
                    &key_name,
                )))?;
            let value: gludconfig::value::Value = ::gludconfig::value::Value::new::<OwnedValue>(
                set_value.into(),
                property.signature(),
            )
            .map_err(|err| zbus::fdo::Error::Failed(format!("{}", err)))?;

            property
                .set_value(value)
                .map_err(|err| zbus::fdo::Error::Failed(format!("{}", err)))?;

            self.storage.update_schema(&schema).await?;
            Self::property_changed(&signal_ctx, schema_name, key_name).await?;
            Ok(())
        }

        #[dbus_interface(name = "reset")]
        async fn reset(
            &self,
            #[zbus(signal_context)] ctx: SignalContext<'_>,
            schema_name: String,
            key_name: String,
        ) -> zbus::fdo::Result<bool> {
            let mut schema = self.storage.get_schema(schema_name.clone()).await?;
            let property = schema
                .properties_mut()
                .find(|p| p.name() == &key_name)
                .ok_or(Into::<zbus::fdo::Error>::into(ZbusError::PropertyNotFound(
                    &schema_name,
                    &key_name,
                )))?;

            let was_reset = property.reset();

            self.storage.update_schema(&schema).await?;
            if was_reset {
                Self::property_changed(&ctx, schema_name, key_name).await?
            };
            Ok(was_reset)
        }

        #[dbus_interface(name = "metadata")]
        async fn metadata(
            &self,
            schema_name: String,
            key_name: String,
        ) -> zbus::fdo::Result<PropertyInfo> {
            let mut schema = self.storage.get_schema(schema_name.clone()).await?;
            let property = schema
                .into_properties()
                .find(|p| p.name() == &key_name)
                .ok_or(Into::<zbus::fdo::Error>::into(ZbusError::PropertyNotFound(
                    &schema_name,
                    &key_name,
                )))?;

            Ok(property.into())
        }
    }

    pub struct Interface {
        pub storage: Storage,
    }

    #[derive(serde::Serialize, serde::Deserialize, zvariant::Type, zvariant::Value)]
    struct PropertyInfo {
        writable: bool,
        name: String,
        description: String,
        summary: String,
        show_in_setitngs: bool,
        signature: OwnedSignature,
        current: Nullable,
    }

    impl From<Property> for PropertyInfo {
        fn from(value: Property) -> Self {
            PropertyInfo {
                name: value.name().to_string(),
                writable: value.is_writable(),
                description: value.long_about().to_string(),
                summary: value.about().to_string(),
                show_in_setitngs: value.show_in_settings(),
                signature: value.signature().into(),
                current: <Property as Into<gludconfig::value::Value>>::into(value).into(),
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, zvariant::Type, zvariant::Value)]
    struct SchemaInfo {
        name: String,
        version: f64,
        triggers: Vec<String>,
        properties: Vec<String>,
    }

    impl From<Schema> for SchemaInfo {
        fn from(value: Schema) -> Self {
            Self {
                version: value.version().into(),
                name: value.name().to_string(),
                triggers: value
                    .triggers()
                    .map(|trigger| trigger.name().to_string())
                    .collect(),
                properties: value
                    .properties()
                    .map(|property| property.name().to_string())
                    .collect(),
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, zvariant::Type, zvariant::Value)]
    struct TriggerInfo {
        name: String,
        trigger: OwnedSignature,
    }

    impl From<Trigger> for TriggerInfo {
        fn from(value: Trigger) -> Self {
            TriggerInfo {
                name: value.name().to_string(),
                trigger: value.signature().into(),
            }
        }
    }

    impl Interface {
        async fn get_schema_mut(&mut self, schema_name: &str) -> zbus::fdo::Result<Schema> {
            let mut schema = self
                .storage
                .get_schema(schema_name.to_string())
                .await
                .map_err(|err| {
                    Into::<zbus::fdo::Error>::into(ZbusError::SchemaNotFound(&schema_name))
                })?;
            Ok(schema)
        }

        async fn sync_db(&mut self, schema: &Schema) -> zbus::fdo::Result<()> {
            self.storage
                .update_schema(&schema)
                .await
                .map_err(|err| zbus::fdo::Error::Failed(format!("{}", err)))
        }
    }

    #[dbus_interface(name = "org.glud.GludConfig.Property")]
    impl Interface {
        async fn register_schema(&mut self, binary: Vec<u8>) -> zbus::fdo::Result<()> {
            let ctx = zvariant::EncodingContext::<byteorder::LE>::new_dbus(0);
            let schema: Schema =
                from_slice(&binary, ctx).map_err(|e| zbus::fdo::Error::Failed(format!("{}", e)))?;
            self.storage
                .new_schema(&schema)
                .await
                .map_err(|e| zbus::fdo::Error::Failed(format!("{}", e)))?;
            Ok(())
        }
        #[dbus_interface(name = "list_schemas")]
        async fn list_schemas(&mut self) -> zbus::fdo::Result<Vec<(String, f32)>> {
            let mut schemas = self
                .storage
                .fetch_all()
                .await
                .map_err(|errr| zbus::fdo::Error::Failed(format!("{}", errr)))?
                .into_iter()
                .map(|schema| (schema.name().to_string(), schema.version()));
            Ok(schemas.collect())
        }
    }
}

#[cfg(feature = "dbus")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::sync::Arc;

    use interface::TriggerInterface;

    use crate::interface::{PropertyInterface, SchemaInterface};

    let storage = Arc::new(gludconfig::storage::Storage::new().await?);
    let connection = zbus::ConnectionBuilder::session()?
        .name("org.glud.GludConfig")?
        .serve_at(
            "/org/glud/gludconfig/property",
            PropertyInterface {
                storage: storage.clone(),
            },
        )?
        .serve_at(
            "/org/glud/gludconfig/trigger",
            TriggerInterface {
                storage: storage.clone(),
            },
        )?
        .serve_at(
            "/org/glud/gludconfig/schema",
            SchemaInterface {
                storage: storage.clone(),
            },
        )?
        .build()
        .await?;

    std::future::pending::<()>().await;
    Ok(())
}

#[cfg(not(feature = "dbus"))]
fn main() {
    panic!("dbus feature is not enabled")
}
