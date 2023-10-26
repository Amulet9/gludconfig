use std::collections::BTreeMap;

use gludconfig::value::Value;
use zvariant::OwnedValue;

fn main() {}

#[cfg(feature = "tests")]
#[derive(serde::Serialize, Debug, serde::Deserialize, zvariant::Type, zvariant::Value)]
enum ScaleMode {
    Loseless = 0,
    Blurry = 1,
}

#[cfg(feature = "tests")]
#[derive(glud_macros::Schema, Debug)]
#[schema(name = "org.desktop.ui.wallpaper", version = 0.1)]
struct WallpaperDaemon {
    #[field(
        name = "wallpaper_path",
        about = "The path of the wallpaper",
        long_about = "The path of the wallpaper that is used by the daemon. It must be one of the supported formats (gif, png, jpeg, svg), otherwise the wallpaper daemon will resort to the default wallpaper and inform the user.",
        writable = true,
        show_in_settings = true,
        default = wallpaper_default,
    )]
    path: String,
    #[field(
        name = "scale_mode",
        about = "the scale mode of the wallpaper",
        show_in_settings = false,
        writable = false
    )]
    scale_mode: ScaleMode,

    #[trigger(name = "tulip")]
    mom: (i32, i32),

    #[field(
        about = "some useless property",
        show_in_settings = false,
        writable = true,
        default = self::default_useless_property,
        choices = self::useless_property_choices
    )]
    some_property: Vec<String>,
}

#[cfg(feature = "tests")]
pub fn useless_property_choices() -> Vec<std::option::Option<Vec<String>>> {
    vec![Some(vec![String::from("foo")])]
}

#[cfg(feature = "tests")]
pub fn default_useless_property() -> Option<Vec<String>> {
    Some(vec![String::from("foo")])
}

#[cfg(feature = "tests")]
pub fn wallpaper_default() -> Option<String> {
    Some(String::from("/usr/share/desktop/ui/defaults/wallpaper.svg"))
}

#[cfg(feature = "tests")]
#[tokio::test]
async fn test_fetch_all() {
    use gludconfig::storage::Storage;

    let mut db = Storage::new().await.unwrap();
    let schemas = db.fetch_all();
    println!("{:#?}", schemas.await.unwrap());
}

#[cfg(feature = "tests")]
#[tokio::test]
async fn test_crud_speed() {
    use gludconfig::storage::Storage;

    let mut k = 0;
    let mut db = Storage::new().await.unwrap();
    for i in 1..10000 {
        let mut schema = db
            .get_schema("org.desktop.ui.wallpaper".to_string())
            .await
            .unwrap();

        schema
            .properties_mut()
            .find(|p| p.name() == "wallpaper_path")
            .unwrap()
            .set_value(gludconfig::value::Value::wrap(Some(format!("{}", i))))
            .unwrap();

        db.update_schema(&schema).await.unwrap();
    }
}

#[cfg(feature = "tests")]
#[tokio::test]
async fn test_read_speed() {
    use gludconfig::storage::Storage;
    use serde::de;
    let mut db = Storage::new().await.unwrap();
    for i in 1..10000 {
        let mut schema = db
            .get_schema("org.desktop.ui.wallpaper".to_string())
            .await
            .unwrap();

        schema
            .properties_mut()
            .find(|p| p.name() == "wallpaper_path")
            .unwrap();
    }
}

#[cfg(feature = "tests")]
#[tokio::test]
async fn test_duplicate() {
    use gludconfig::{schema, storage::Storage};

    let schema = WallpaperDaemon::schema().unwrap();
    let mut db = Storage::new().await.unwrap();
    assert_eq!(
        db.new_schema(&schema).await.is_ok(),
        true,
        "The schema likely already exists on your system, please delete it before proceeding."
    );
    assert_eq!(db.new_schema(&schema).await.is_ok(), false);
}

#[cfg(feature = "tests")]
#[tokio::test]
async fn test_homepath() {
    let homepath = homedir::get_my_home().unwrap().unwrap();
    println!("{:#?}", homepath);
}

#[cfg(feature = "tests")]
#[tokio::test]
async fn test_fetch_schema() {
    use gludconfig::storage::Storage;

    let mut db = Storage::new().await.unwrap();
    let schema = db.get_schema("org.desktop.ui.wallpaper".to_string()).await;

    assert_eq!(schema.is_ok(), true);
}

#[cfg(feature = "tests")]
#[tokio::test]
async fn test_update_writable_property() {
    use gludconfig::{storage::Storage, value::*};
    let mut db = Storage::new().await.unwrap();
    let mut schema = db
        .get_schema("org.desktop.ui.wallpaper".to_string())
        .await
        .unwrap();

    let mut property = schema
        .properties_mut()
        .find(|p| p.name() == "wallpaper_path")
        .unwrap();

    assert_eq!(
        property
            .set_value(Value::wrap(Some("".to_string())))
            .is_ok(),
        true
    );

    assert_eq!(property.set_value(Value::wrap(Some(500))).is_ok(), false);
}

#[cfg(feature = "tests")]
#[tokio::test]
async fn test_signal() {
    use gludconfig::{storage::Storage, value::*};
    use zvariant::OwnedValue;
    let mut db = Storage::new().await.unwrap();
    let mut schema = db
        .get_schema("org.desktop.ui.wallpaper".to_string())
        .await
        .unwrap();

    let trigger = schema.triggers_mut().find(|t| t.name() == "tulip");

    assert_eq!(trigger.is_some(), true);

    let trigger = trigger.unwrap();

    assert_eq!(trigger.matches(&zvariant::Value::from((5, 3)).into()), true);
    assert_eq!(trigger.matches(&zvariant::Value::from(5).into()), false);
}

#[cfg(feature = "tests")]
#[tokio::test]
async fn test_update_unwritable_property() {
    use gludconfig::{storage::Storage, value::*};
    let mut db = Storage::new().await.unwrap();
    let mut schema = db
        .get_schema("org.desktop.ui.wallpaper".to_string())
        .await
        .unwrap();

    let mut property = schema
        .properties_mut()
        .find(|p| p.name() == "scale_mode")
        .unwrap();
    assert_eq!(property.set_value(Value::wrap(Some(1))).is_ok(), false);
}

#[cfg(feature = "tests")]
#[tokio::test]
async fn test_choices_property() {
    use gludconfig::{storage::Storage, value::*};
    let mut db = Storage::new().await.unwrap();
    let mut schema = db
        .get_schema("org.desktop.ui.wallpaper".to_string())
        .await
        .unwrap();

    let mut property = schema
        .properties_mut()
        .find(|p| p.name() == "some_property")
        .unwrap();

    assert_eq!(
        property
            .set_value(Value::wrap(Some(vec![String::from("not_foo")])))
            .is_ok(),
        false
    );

    assert_eq!(
        property
            .set_value(Value::wrap::<Vec<String>>(Some(vec![String::from("foo")])))
            .is_ok(),
        true
    )
}

#[cfg(all(feature = "dbus", feature = "tests", feature = "macros"))]
async fn test_generate_async() {
    use futures_util::StreamExt;

    let conn = zbus::Connection::session().await.unwrap();

    #[glud_macros::glud_interface(name = "org.desktop.ui.wallpaper", blocking = false)]
    trait WallpaperDaemon {
        #[property(name = "wallpaper_path")]
        pub async fn wallpaper_path() -> String;
        #[property(name = "scale_mode")]
        pub async fn scale_mode() -> u32;
        #[property(name = "some_property")]
        pub async fn some_property() -> Vec<String>;
        #[trigger(name = "tulip")]
        pub async fn tulip() -> (i32, i32);
    }

    let daemon = WallpaperDaemon::new(&conn).await.unwrap();
    // let mut stream = daemon.tulip_occured().await.unwrap();

    // while let Some(event) = stream.next().await {
        // println!("{}", event);
    // }
}
