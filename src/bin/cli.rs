use std::fmt::{Debug, Display};

#[cfg(feature = "cli")]
mod trigger {
    use zbus::dbus_proxy;

    #[dbus_proxy(
        interface = "org.glud.GludConfig.Trigger",
        default_service = "org.glud.GludConfig",
        default_path = "/org/glud/gludconfig/trigger"
    )]
    trait Trigger {
        /// invoke_trigger method
        #[dbus_proxy(name = "invoke_trigger")]
        fn invoke_trigger(
            &self,
            schema_name: &str,
            trigger_name: &str,
            value: &zbus::zvariant::Value<'_>,
        ) -> zbus::Result<()>;

        /// metadata method
        #[dbus_proxy(name = "metadata")]
        fn metadata(
            &self,
            schema_name: &str,
            trigger_name: &str,
        ) -> zbus::Result<(String, zbus::zvariant::OwnedSignature)>;

        /// trigger_invoked signal
        #[dbus_proxy(signal, name = "trigger_invoked")]
        fn trigger_invoked(
            &self,
            schema_name: &str,
            trigger_name: &str,
            value: zbus::zvariant::Value<'_>,
        ) -> zbus::Result<()>;
    }
}
#[cfg(feature = "cli")]
mod schema {
    use zbus::dbus_proxy;

    #[dbus_proxy(
        interface = "org.glud.GludConfig.Schema",
        default_service = "org.glud.GludConfig",
        default_path = "/org/glud/gludconfig/schema"
    )]
    trait Schema {
        /// all method
        #[dbus_proxy(name = "all")]
        fn all(&self) -> zbus::Result<Vec<(String, u32, Vec<String>, Vec<String>)>>;

        /// metadata method
        #[dbus_proxy(name = "metadata")]
        fn metadata(
            &self,
            schema_name: &str,
        ) -> zbus::Result<(String, u32, Vec<String>, Vec<String>)>;

        /// register method
        #[dbus_proxy(name = "register")]
        fn register(&self, data: &[u8]) -> zbus::Result<()>;

        /// reset_all method
        #[dbus_proxy(name = "reset_all")]
        fn reset_all(&self, schema_name: &str) -> zbus::Result<bool>;
    }
}

#[cfg(feature = "cli")]
mod property {

    use zbus::dbus_proxy;

    #[dbus_proxy(
        interface = "org.glud.GludConfig.Property",
        default_service = "org.glud.GludConfig",
        default_path = "/org/glud/gludconfig/property"
    )]
    trait Property {
        /// metadata method
        #[dbus_proxy(name = "metadata")]
        fn metadata(
            &self,
            schema_name: &str,
            key_name: &str,
        ) -> zbus::Result<(
            bool,
            String,
            String,
            String,
            bool,
            zbus::zvariant::OwnedSignature,
            (bool, zbus::zvariant::OwnedValue),
        )>;

        /// reset method
        #[dbus_proxy(name = "reset")]
        fn reset(&self, schema_name: &str, key_name: &str) -> zbus::Result<bool>;

        /// set method
        #[dbus_proxy(name = "set")]
        fn set(
            &self,
            schema_name: &str,
            key_name: &str,
            set_value: &(bool, zbus::zvariant::Value<'_>),
        ) -> zbus::Result<()>;

        /// property_changed signal
        #[dbus_proxy(signal, name = "property_changed")]
        fn property_changed(&self, schema_name: &str, key_name: &str) -> zbus::Result<()>;
    }
}

#[cfg(feature = "cli")]
mod cli {
    use bpaf::*;
    use zvariant::OwnedValue;

    use crate::{property, schema, trigger};

    #[derive(Debug, Clone, Bpaf)]
    pub enum GludCli {
        #[bpaf(command("info"))]
        Info {
            #[bpaf(positional("SCHEMA_NAME"))]
            schema_name: String,
            #[bpaf(positional("PROPERTY_NAME"))]
            property_name: String,
        },
        #[bpaf(command("reset"))]
        Reset {
            #[bpaf(positional("SCHEMA_NAME"))]
            schema_name: String,
            #[bpaf(positional("PROPERTY_NAME"))]
            property_name: String,
        },
        #[bpaf(command("monitor"))]
        Monitor {
            #[bpaf(short, long)]
            trigger: bool,
            #[bpaf(positional("SCHEMA_NAME"))]
            schema_name: String,
            #[bpaf(positional("PROPERTY_NAME"))]
            name: String,
        },
        #[bpaf(command("reset-recursively"))]
        ResetRecursively {
            #[bpaf(positional("SCHEMA_NAME"))]
            schema_name: String,
        },
        #[bpaf(command("list-schemas"))]
        ListSchemas,
    }

    pub async fn list_schemas(conn: &zbus::Connection) -> anyhow::Result<String> {
        let proxy = schema::SchemaProxy::new(&conn).await?;
        let schemas = proxy.all().await?;
        let json = serde_json::json!({
            "schemas": schemas,
        });
        return Ok(serde_json::to_string_pretty(&json)?);
    }

    pub async fn reset_recursively(
        schema_name: String,
        conn: &zbus::Connection,
    ) -> anyhow::Result<String> {
        let proxy = schema::SchemaProxy::new(&conn).await?;
        let success = proxy.reset_all(&schema_name).await?;
        let value = serde_json::json!({
            "success": success,
        });
        let val = serde_json::to_string_pretty(&value)?;
        Ok(val)
    }

    pub async fn reset(
        schema_name: String,
        property_name: String,
        conn: &zbus::Connection,
    ) -> anyhow::Result<String> {
        let proxy = property::PropertyProxy::new(&conn).await?;
        let success = proxy.reset(&schema_name, &property_name).await?;

        let value = serde_json::json!({
            "success": success,
        });

        let val = serde_json::to_string_pretty(&value)?;
        Ok(val)
    }

    fn convert_property_to_serde(value: (bool, OwnedValue)) -> anyhow::Result<serde_json::Value> {
        if value.0 {
            Ok(serde_json::Value::Null)
        } else {
            Ok(serde_json::to_value(value.1)?)
        }
    }

    pub async fn monitor(
        schema_name: String,
        name: String,
        trigger: bool,
        conn: &zbus::Connection,
    ) -> anyhow::Result<String> {
        use futures_util::StreamExt;

        if !trigger {
            let proxy = property::PropertyProxy::new(&conn).await?;
            let mut signal: property::property_changedStream<'_> = proxy
                .receive_property_changed_with_args(&[(0, &schema_name), (1, &name)])
                .await?;

            let mut current_property =
                convert_property_to_serde(proxy.metadata(&schema_name, &name).await?.6)?;

            while let Some(change) = signal.next().await {
                let new_value =
                    convert_property_to_serde(proxy.metadata(&schema_name, &name).await?.6)?;
                let json = serde_json::json!({
                    "schema": &schema_name,
                    "property": &name,
                    "from": &current_property,
                    "to": &new_value,
                });
                current_property = new_value;
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
        } else {
            let mut proxy = trigger::TriggerProxy::new(&conn).await?;
            let mut trigger = proxy
                .receive_trigger_invoked_with_args(&[(0, &schema_name), (1, &name)])
                .await?;
            let mut metadata = proxy.metadata(&schema_name, &name).await?;

            while let Some(trigger) = trigger.next().await {
                let value = serde_json::to_value(trigger.args()?.value())?;
                let json = serde_json::json!({
                    "schema": &schema_name,
                    "trigger": &name,
                    "signature": &metadata.1,
                    "args": value,
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
        }
        anyhow::bail!("Signal Stream has ended")
    }

    pub async fn info(
        schema_name: String,
        property_name: String,
        conn: &zbus::Connection,
    ) -> anyhow::Result<String> {
        let proxy = property::PropertyProxy::new(&conn).await?;
        let info = proxy.metadata(&schema_name, &property_name).await?;

        let current = convert_property_to_serde(info.6)?;

        let value = serde_json::json!({
            "name": info.1,
            "writable": info.0,
            "about": info.3,
            "long_about": info.2,
            "sos": info.4,
            "signature": info.5,
            "value": current,
        });

        let val = serde_json::to_string_pretty(&value)?;
        Ok(val)
    }
}

#[cfg(feature = "cli")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    use bpaf::Parser;
    use cli::glud_cli;

    let conn = zbus::Connection::session().await?;
    let output = match glud_cli().run() {
        cli::GludCli::Info {
            schema_name,
            property_name,
        } => map_err_to_str(cli::info(schema_name, property_name, &conn).await),
        cli::GludCli::Reset {
            schema_name,
            property_name,
        } => map_err_to_str(cli::reset(schema_name, property_name, &conn).await),
        cli::GludCli::Monitor {
            schema_name,
            name,
            trigger,
        } => map_err_to_str(cli::monitor(schema_name, name, trigger, &conn).await),
        cli::GludCli::ResetRecursively { schema_name } => {
            map_err_to_str(cli::reset_recursively(schema_name, &conn).await)
        }
        cli::GludCli::ListSchemas => map_err_to_str(cli::list_schemas(&conn).await),
    };

    println!("{}", output);
    Ok(())
}

fn map_err_to_str<T: Display, E: Display>(err: Result<T, E>) -> String {
    match err {
        Ok(val) => format!("{}", val),
        Err(err) => format!("{}", err),
    }
}

#[cfg(not(feature = "cli"))]
fn main() {
    panic!("Cli or dbus werent enabled while compiling")
}
