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
    use clap::{Parser, Subcommand};
    use futures_util::StreamExt;
    use zvariant::OwnedValue;

    use crate::{property, schema, trigger};

    #[derive(Parser)]
    #[command(
        author = "gludconfig",
        version,
        name = "gludconfig",
        about = "CLI Tool to interact with the gludconfig dbus daemon"
    )]
    pub enum GludCli {
        #[command(subcommand)]
        Property(PropertyCommand),
        #[command(subcommand)]
        Schema(SchemaCommand),
        #[command(subcommand)]
        TriggerCommand(TriggerCommand),
    }

    #[derive(Subcommand)]
    #[command(
        name = "trigger",
        author = "gludconfig",
        version,
        about = "Commands releated to triggers"
    )]
    pub enum TriggerCommand {
        #[command(
            name = "monitor",
            author = "gludconfig",
            version,
            about = "Monitor triggers being triggered"
        )]
        Monitor {
            schema_name: String,
            trigger_name: String,
        },
        #[command(
            name = "metadata",
            author = "gludconfig",
            version,
            about = "Provides metadata about a trigger, such as its signature"
        )]
        Metadata {
            schema_name: String,
            trigger_name: String,
        },
    }

    #[derive(Subcommand)]
    #[command(
        name = "schema",
        author = "gludconfig",
        version,
        about = "Commands releated to schemas"
    )]
    pub enum SchemaCommand {
        #[command(
            name = "list-schemas",
            author = "gludconfig",
            version,
            about = "List all schemas along with their property and trigger names"
        )]
        ListAll,

        #[command(
            name = "metadata",
            author = "gludconfig",
            version,
            about = "Get metadata about a schema! Including its property and trigger names!"
        )]
        Metadata { schema_name: String },

        #[command(
            name = "reset-recursively",
            author = "gludconfig",
            version,
            about = "Reset all values in a schema recursively",
            long_about = "Reset all values in a schema recursively! Returns false even if one of the keys is not writable!"
        )]
        ResetRecursively { schema_name: String },
    }

    #[derive(Subcommand)]
    #[command(
        name = "property",
        author = "gludconfig",
        version,
        about = "Commands releated to properties"
    )]
    pub enum PropertyCommand {
        #[command(
            author = "gludconfig",
            version,
            name = "metadata",
            about = "Get metadata for a property(including its current value). In Json Format"
        )]
        Metadata {
            schema_name: String,
            property_name: String,
        },
        #[command(
            author = "gludconfig",
            name = "reset",
            version,
            about = "Reset a property to its default value",
            long_about = "Reset a property to its default value! Fails if property is not writable! If no default value is there, it will reset to `null`"
        )]
        Reset {
            schema_name: String,
            property_name: String,
        },

        #[command(
            author = "gludconfig",
            name = "monitor",
            version,
            about = "Monitor a property for changes!"
        )]
        Monitor {
            schema_name: String,
            property_name: String,
        },
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

    pub async fn monitor_property(
        schema_name: String,
        property_name: String,
        conn: &zbus::Connection,
    ) -> anyhow::Result<String> {
        use futures_util::StreamExt;
        let proxy = property::PropertyProxy::new(&conn).await?;
        let mut signal: property::property_changedStream<'_> = proxy
            .receive_property_changed_with_args(&[(0, &schema_name), (1, &property_name)])
            .await?;

        let mut current_property =
            convert_property_to_serde(proxy.metadata(&schema_name, &property_name).await?.6)?;

        while let Some(change) = signal.next().await {
            let new_value =
                convert_property_to_serde(proxy.metadata(&schema_name, &property_name).await?.6)?;
            let json = serde_json::json!({
                "schema": &schema_name,
                "property": &property_name,
                "from": &current_property,
                "to": &new_value,
            });
            current_property = new_value;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }

        anyhow::bail!("Property Stream has ended")
    }

    pub async fn montior_trigger(
        schema_name: String,
        trigger_name: String,
        conn: &zbus::Connection,
    ) -> anyhow::Result<String> {
        let mut proxy = trigger::TriggerProxy::new(&conn).await?;
        let mut trigger = proxy
            .receive_trigger_invoked_with_args(&[(0, &schema_name), (1, &trigger_name)])
            .await?;
        let mut metadata = proxy.metadata(&schema_name, &trigger_name).await?;

        while let Some(trigger) = trigger.next().await {
            let value = serde_json::to_value(trigger.args()?.value())?;
            let json = serde_json::json!({
                "schema": &schema_name,
                "trigger": &trigger_name,
                "signature": &metadata.1,
                "args": value,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        anyhow::bail!("Trigger steam has ended")
    }

    pub async fn metadata_schema(
        schema_name: String,
        conn: &zbus::Connection,
    ) -> anyhow::Result<String> {
        let proxy = schema::SchemaProxy::new(&conn).await?;
        let metadata = proxy.metadata(&schema_name).await?;

        let json = serde_json::json!({
            "name": &metadata.0,
            "version": &metadata.1,
            "triggers": &metadata.2,
            "properties": &metadata.3,
        });

        Ok(serde_json::to_string_pretty(&json)?)
    }

    pub async fn metadata_trigger(
        schema_name: String,
        trigger_name: String,
        conn: &zbus::Connection,
    ) -> anyhow::Result<String> {
        let mut proxy = trigger::TriggerProxy::new(&conn).await?;
        let mut metadata = proxy.metadata(&schema_name, &trigger_name).await?;
        let json = serde_json::json!({
            "name": &metadata.0,
            "signature": &metadata.1,
        });

        return Ok(serde_json::to_string_pretty(&json)?);
    }

    pub async fn metadata_property(
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
    use clap::Parser;

    let conn = zbus::Connection::session().await?;
    let cmd = cli::GludCli::parse();

    let output = match cmd {
        cli::GludCli::Property(cmd) => match cmd {
            cli::PropertyCommand::Metadata {
                schema_name,
                property_name,
            } => map_err_to_str(cli::metadata_property(schema_name, property_name, &conn).await),
            cli::PropertyCommand::Reset {
                schema_name,
                property_name,
            } => map_err_to_str(cli::reset(schema_name, property_name, &conn).await),
            cli::PropertyCommand::Monitor {
                schema_name,
                property_name,
            } => map_err_to_str(cli::monitor_property(schema_name, property_name, &conn).await),
        },
        cli::GludCli::Schema(cmd) => match cmd {
            cli::SchemaCommand::ListAll => map_err_to_str(cli::list_schemas(&conn).await),
            cli::SchemaCommand::Metadata { schema_name } => {
                map_err_to_str(cli::metadata_schema(schema_name, &conn).await)
            }
            cli::SchemaCommand::ResetRecursively { schema_name } => {
                map_err_to_str(cli::reset_recursively(schema_name, &conn).await)
            }
        },
        cli::GludCli::TriggerCommand(cmd) => match cmd {
            cli::TriggerCommand::Monitor {
                schema_name,
                trigger_name,
            } => map_err_to_str(cli::montior_trigger(schema_name, trigger_name, &conn).await),
            cli::TriggerCommand::Metadata {
                schema_name,
                trigger_name,
            } => map_err_to_str(cli::metadata_trigger(schema_name, trigger_name, &conn).await),
        },
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
