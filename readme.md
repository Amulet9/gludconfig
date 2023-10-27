# GludConfig

GludConfig is a attempt to rewrite GSettings in rust with additional features. Since this project was made in a rush, the code structure is not exaclty in the uhm, _best condition_, however I will try my best to update it accordingly in the future.

## Features

- [x] Sqlite Backend
- [x] Automatic Code Generation
- [x] Macro-Based Schema Definition
- [x] DBus Interface
- [ ] Protection Against External Changes (TODO, Planned)
- [ ] Cli Interface (TODO, Planned)
- [x] Triggers/Signals
- [ ] Enum Support: _Although custom enums can be used through zvariant and serde magic, first-class support is still not there._ (TODO).
- [ ] Interop with GSettings (Likely Never)
- [ ] Vendor Overrides (Likely Never)

# How it works

Upon starting the dbus daemon, through the main user, it connects to `.local/share/gludconfig/data.db`. Whenever another program wants to use the daemon, it first must try to fetch its schema, if it doesent exist, then try to register it. To generate interfacing code, look at `bin/generate_code.rs`

# Known Issues

- [ ] Awful code structure
- [ ] Not user friendly enough, doesent provide service files or anything.
- [ ] Code generation doesent auto deserialize signal body into the provided type. [Possible Fix: Generate custom struct per Trigger/Property]

# Register Schema

```rust
    #[derive(glud_macros::Schema, Debug)]
    #[schema(name = "org.foo.foo", version = 0.1)]
    struct Foo {
    #[field(
        name = "foo_another",
        writable = false,
        about = "about",
        long_about = "long about",
        show_in_settings = true
    )]
    foo: u32,

    #[field(default = default_with_choices, choices = choices)]
    with_choices: i32,
}

fn default_with_choices() -> Option<i32> {
    Some(5)
}

fn choices() -> Vec<Option<i32>> {
    vec![Some(5), Some(9), Some(10)]
}



#[tokio::main]
async fn main() -> ::gsd_rs::Result<()> {
    let schema = Foo:schema()?;

    // let conn = zbus::blocking::Connection::session()?;
    // Foo::register_sync(&schema, &conn)?;

    let conn = zbus::Connection::session().await?;
    Foo::register_async(&schema, &conn).await?;
    Ok(())
}
```

# Using the daemon

Use

```bash
[path-to-generate-code-binary] org.foo.foo Foo
```

> tip: use --blocking if you want to use `zbus::blocking`

Should generate someting like

```rust
#[glud_macros::glud_interface(name = "org.foo.foo", blocking = false)]
trait Foo {
    #[property(name = "foo_another")]
    async fn foo_another() -> u32;
    #[property(name = "with_choices")]
    async fn with_choices() -> i32;
}
```

this should generate the proper "reset", "read", "info", "changed" and "set" methods for the properties.
