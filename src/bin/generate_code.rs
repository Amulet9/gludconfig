#[cfg(feature = "cli")]
mod gen_code {
    use gludconfig::schema::Schema;
    use zvariant::Basic;
    use zvariant::Signature;

   

    #[derive(bpaf::Bpaf, Debug, Clone)]
    
    pub struct Gencode {
        #[bpaf(short, long)]
        pub blocking: bool,
        #[bpaf(positional("SCHEMA"))]
        pub schema: String,
        #[bpaf(positional("STRUCT_NAME"))]
        pub name: String,
    }

    fn to_rust_type(ty: &Signature<'static>, input: bool, as_ref: bool) -> String {
        fn iter_to_rust_type(
            it: &mut std::iter::Peekable<std::slice::Iter<'_, u8>>,
            input: bool,
            as_ref: bool,
        ) -> String {
            let c = it.next().unwrap();
            match *c as char {
                u8::SIGNATURE_CHAR => "u8".into(),
                bool::SIGNATURE_CHAR => "bool".into(),
                i16::SIGNATURE_CHAR => "i16".into(),
                u16::SIGNATURE_CHAR => "u16".into(),
                i32::SIGNATURE_CHAR => "i32".into(),
                u32::SIGNATURE_CHAR => "u32".into(),
                i64::SIGNATURE_CHAR => "i64".into(),
                u64::SIGNATURE_CHAR => "u64".into(),
                f64::SIGNATURE_CHAR => "f64".into(),
                // xmlgen accepts 'h' on Windows, only for code generation
                'h' => (if input {
                    "zbus::zvariant::Fd"
                } else {
                    "zbus::zvariant::OwnedFd"
                })
                .into(),
                <&str>::SIGNATURE_CHAR => (if input || as_ref { "&str" } else { "String" }).into(),
                zvariant::ObjectPath::SIGNATURE_CHAR => (if input {
                    if as_ref {
                        "&zbus::zvariant::ObjectPath<'_>"
                    } else {
                        "zbus::zvariant::ObjectPath<'_>"
                    }
                } else {
                    "zbus::zvariant::OwnedObjectPath"
                })
                .into(),
                zvariant::Signature::SIGNATURE_CHAR => (if input {
                    if as_ref {
                        "&zbus::zvariant::Signature<'_>"
                    } else {
                        "zbus::zvariant::Signature<'_>"
                    }
                } else {
                    "zbus::zvariant::OwnedSignature"
                })
                .into(),
                zvariant::VARIANT_SIGNATURE_CHAR => (if input {
                    if as_ref {
                        "&zbus::zvariant::Value<'_>"
                    } else {
                        "zbus::zvariant::Value<'_>"
                    }
                } else {
                    "zbus::zvariant::OwnedValue"
                })
                .into(),
                zvariant::ARRAY_SIGNATURE_CHAR => {
                    let c = it.peek().unwrap();
                    match **c as char {
                        '{' => format!(
                            "std::collections::HashMap<{}>",
                            iter_to_rust_type(it, input, false)
                        ),
                        _ => {
                            let ty = iter_to_rust_type(it, input, false);
                            if input {
                                format!("&[{ty}]")
                            } else {
                                format!("{}Vec<{}>", if as_ref { "&" } else { "" }, ty)
                            }
                        }
                    }
                }
                c @ zvariant::STRUCT_SIG_START_CHAR | c @ zvariant::DICT_ENTRY_SIG_START_CHAR => {
                    let dict = c == '{';
                    let mut vec = vec![];
                    loop {
                        let c = it.peek().unwrap();
                        match **c as char {
                            zvariant::STRUCT_SIG_END_CHAR | zvariant::DICT_ENTRY_SIG_END_CHAR => {
                                break
                            }
                            _ => vec.push(iter_to_rust_type(it, input, false)),
                        }
                    }
                    if dict {
                        vec.join(", ")
                    } else if vec.len() > 1 {
                        format!("{}({})", if as_ref { "&" } else { "" }, vec.join(", "))
                    } else {
                        vec[0].to_string()
                    }
                }
                _ => unimplemented!(),
            }
        }

        let mut it = ty.as_bytes().iter().peekable();
        iter_to_rust_type(&mut it, input, as_ref)
    }

    pub fn generate_for_schema(
        schema: &mut Schema,
        name: &str,
        blocking: bool,
        write: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        writeln!(
            write,
            "#[glud_macros::glud_interface(name = \"{}\", blocking = {})]",
            schema.name(),
            blocking
        )?;
        writeln!(write, "trait {} {{", name)?;

        for property in schema.properties() {
            writeln!(write, "    #[property(name = \"{}\")]", property.name())?;
            let _async = blocking.then(|| "").unwrap_or("async");
            writeln!(
                write,
                "    {} fn {}() -> {};",
                _async,
                property.name(),
                to_rust_type(&property.signature(), false, false)
            )?;
        }

        for trigger in schema.triggers() {
            writeln!(write, "    #[trigger(name = \"{}\")]", trigger.name())?;
            let _async = blocking.then(|| "").unwrap_or("async");
            writeln!(
                write,
                "    {} fn {}() -> {};",
                _async,
                trigger.name(),
                to_rust_type(&trigger.signature(), false, false)
            )?;
        }
        writeln!(write, "}}")?;
        Ok(())
    }
}

#[cfg(feature = "cli")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    use bpaf::Parser;
    use gludconfig::storage::Storage;

    let options = gen_code::gencode().run();
    let mut storage = Storage::new().await?;
    let mut schema = storage.get_schema(options.schema).await?;

    gen_code::generate_for_schema(
        &mut schema,
        &options.name,
        options.blocking,
        &mut std::io::stdout(),
    )?;

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    panic!("Code compiled without dbus or cli feature")
}
