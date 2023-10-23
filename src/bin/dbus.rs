#[cfg(feature = "dbus")]
mod interface {
    use std::collections::BTreeMap;

    use gsd_rs::{
        error::ZbusError, property::Property, schema::Schema, storage::Storage, trigger::Trigger,
        value::Nullable,
    };
    use zbus::{dbus_interface, SignalContext};
    use zvariant::{from_slice, OwnedSignature, OwnedValue, Signature};

    pub struct Interface {
        pub storage: Storage,
    }

    #[derive(serde::Serialize, serde::Deserialize, zvariant::Type, zvariant::Value)]
    struct PropertyInfo {
        writable: bool,
        description: String,
        summary: String,
        show_in_setitngs: bool,
        signature: OwnedSignature,
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

    #[dbus_interface(name = "org.glud.GludConfig")]
    impl Interface {
        async fn trigger(
            &mut self,
            #[zbus(signal_context)] ctx: SignalContext<'_>,
            schema_name: String,
            trigger_name: String,
            value: OwnedValue,
        ) -> zbus::fdo::Result<()> {
            let mut schema = self.get_schema_mut(&schema_name).await?;
            let trigger =
                schema
                    .get_trigger_mut(&trigger_name)
                    .ok_or(Into::<zbus::fdo::Error>::into(ZbusError::TriggerNotFound(
                        &schema_name,
                        &trigger_name,
                    )))?;
            if trigger.matches(&value) {
                Self::trigger_invoked(&ctx, schema_name, value).await?;
            } else {
                return Err(zbus::fdo::Error::Failed(format!(
                    "The signature of the trigger and the provided value dont match"
                )));
            }
            Ok(())
        }

        #[dbus_interface(signal)]
        async fn trigger_invoked(
            ctx: &SignalContext<'_>,
            signal: String,
            value: OwnedValue,
        ) -> zbus::Result<()>;

        #[dbus_interface(signal)]
        async fn property_changed(
            ctx: &SignalContext<'_>,
            schema_name: String,
            key_name: String,
        ) -> zbus::Result<()>;

        async fn set(
            &mut self,
            #[zbus(signal_context)] signal_ctx: SignalContext<'_>,
            schema_name: String,
            key_name: String,
            set_value: Nullable,
        ) -> zbus::fdo::Result<()> {
            let mut schema = self.get_schema_mut(&schema_name).await?;
            let property =
                schema
                    .get_property_mut(&key_name)
                    .ok_or(Into::<zbus::fdo::Error>::into(ZbusError::PropertyNotFound(
                        &schema_name,
                        &key_name,
                    )))?;
            let value: gsd_rs::value::Value =
                ::gsd_rs::value::Value::new::<OwnedValue>(set_value.into(), property.signature())
                    .map_err(|err| zbus::fdo::Error::Failed(format!("{}", err)))?;

            property
                .set_value(value)
                .map_err(|err| zbus::fdo::Error::Failed(format!("{}", err)))?;

            self.sync_db(&schema).await?;
            Self::property_changed(&signal_ctx, schema_name, key_name).await?;
            Ok(())
        }

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

        async fn info(
            &mut self,
            schema_name: String,
            key_name: String,
        ) -> zbus::fdo::Result<PropertyInfo> {
            let mut schema = self.get_schema_mut(&schema_name).await?;
            let property =
                schema
                    .get_property_mut(&key_name)
                    .ok_or(Into::<zbus::fdo::Error>::into(ZbusError::PropertyNotFound(
                        &schema_name,
                        &key_name,
                    )))?;
            Ok(PropertyInfo {
                writable: property.is_writable(),
                description: property.long_about().to_string(),
                summary: property.about().to_string(),
                show_in_setitngs: property.show_in_settings(),
                signature: property.signature().into(),
            })
        }

        async fn read(
            &mut self,
            schema_name: String,
            key_name: String,
        ) -> zbus::fdo::Result<Nullable> {
            let mut schema = self.get_schema_mut(&schema_name).await?;
            let property = schema
                .into_properties()
                .find(|p| p.name() == key_name)
                .ok_or(zbus::fdo::Error::Failed(format!(
                    "{}",
                    ZbusError::PropertyNotFound(&schema_name, &key_name)
                )))?;
            Ok(property.into_value().into_nullabe())
        }

        async fn reset(
            &mut self,
            #[zbus(signal_context)] ctx: SignalContext<'_>,
            schema_name: String,
            key_name: String,
        ) -> zbus::fdo::Result<bool> {
            let mut schema = self.get_schema_mut(&schema_name).await?;
            let property =
                schema
                    .get_property_mut(&key_name)
                    .ok_or(Into::<zbus::fdo::Error>::into(ZbusError::PropertyNotFound(
                        &schema_name,
                        &key_name,
                    )))?;
            let was_reset = property.reset();
            self.sync_db(&schema).await?;
            if was_reset {
                Self::property_changed(&ctx, schema_name, key_name).await?
            };
            Ok(was_reset)
        }

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

        async fn reset_all(
            &mut self,
            #[zbus(signal_context)] ctx: SignalContext<'_>,
            schema_name: String,
        ) -> zbus::fdo::Result<bool> {
            let mut schema = self.get_schema_mut(&schema_name).await?;
            let mut _bool = true;

            for property in schema.iter_properties_mut() {
                if !property.reset() {
                    _bool = false;
                } else {
                    Self::property_changed(
                        &ctx,
                        schema_name.to_string(),
                        property.name().to_string(),
                    )
                    .await?;
                }
            }
            self.sync_db(&schema).await?;
            Ok(_bool)
        }
    }
}

#[cfg(feature = "dbus")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hello = 5;
    let mut connection = gsd_rs::storage::Storage::new().await?;
    let connection = zbus::ConnectionBuilder::session()?
        .name("org.glud.GludConfig")?
        .serve_at(
            "/org/glud/gludconfig",
            interface::Interface {
                storage: connection,
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
