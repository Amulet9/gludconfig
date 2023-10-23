use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro::Span;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, OptionExt};
use quote::ToTokens;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput};
use syn::{Field, Fields};

#[derive(darling::FromDeriveInput)]
#[darling(attributes(schema))]
struct SchemaInput {
    name: String,
    version: f32,
}

#[derive(FromField, Debug)]
#[darling(attributes(trigger))]
struct TriggerInput {
    #[darling(default)]
    name: Option<String>,
    ty: syn::Type,
    ident: Option<syn::Ident>,
}

#[derive(FromField, Debug)]
#[darling(attributes(field))]
struct PropertyInput {
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    about: Option<String>,
    #[darling(default)]
    long_about: Option<String>,
    #[darling(default)]
    writable: Option<bool>,
    #[darling(default)]
    show_in_settings: Option<bool>,
    #[darling(default)]
    default: Option<syn::Path>,
    #[darling(default)]
    choices: Option<syn::Path>,
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    value: Option<syn::Path>,
}

pub fn expand(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);

    let options = match get_options(&input) {
        Ok(e) => e,
        Err(e) => return e,
    };

    let _struct = match get_struct(&input) {
        Ok(e) => e,
        Err(e) => return e,
    };

    let (properties, triggers) = match get_properties(&_struct) {
        Ok(e) => e,
        Err(e) => return e,
    };

    let properties = properties.into_iter().map(generate_for_property);
    let triggers = triggers.into_iter().map(generate_for_trigger);

    let ident = &input.ident;
    let schema = generate_schema(properties, triggers, options);
    let stream = quote::quote!(
        impl #ident {
            pub fn schema() -> ::gsd_rs::Result<::gsd_rs::schema::Schema>  {
                #schema
            }
            pub async fn register_async(schema: &::gsd_rs::schema::Schema, conn: &::zbus::Connection) -> ::gsd_rs::Result<()> {
                let proxy = ::zbus::Proxy::new(conn, "org.glud.GludConfig", "/org/glud/gludconfig", "org.glud.GludConfig").await?;
                let ctx = ::zbus::zvariant::EncodingContext::<::byteorder::LE>::new_dbus(0);
                let bytes = ::zbus::zvariant::to_bytes(ctx, schema)?;
                proxy.call::<_, _, ()>("RegisterSchema", &(bytes)).await?;
                Ok(())
            }
            pub fn register_sync(schema: &::gsd_rs::schema::Schema, conn: &::zbus::blocking::Connection) -> ::gsd_rs::Result<()> {
                let proxy = ::zbus::blocking::Proxy::new(conn, "org.glud.GludConfig", "/org/glud/gludconfig", "org.glud.GludConfig")?;
                let ctx = ::zbus::zvariant::EncodingContext::<::byteorder::LE>::new_dbus(0);
                let bytes = ::zbus::zvariant::to_bytes(ctx, schema)?;
                proxy.call::<_, _, ()>("RegisterSchema", &(bytes))?;
                Ok(())
            }
        }
    );
    stream.into()
}

fn get_struct(input: &DeriveInput) -> Result<DataStruct, proc_macro::TokenStream> {
    match &input.data {
        syn::Data::Struct(str) => Ok(str.clone()),
        _ => {
            abort!(Span::call_site(), "Expected Struct, Found Enum or Union!"; help = "It looks like you have tried to use this macro on a non-struct, which is supported. Perhaps try a struct?")
        }
    }
}

fn get_options(input: &DeriveInput) -> Result<SchemaInput, proc_macro::TokenStream> {
    match SchemaInput::from_derive_input(input) {
        Ok(input) => Ok(input),
        Err(err) => {
            abort!(Span::call_site(), err; help = "The arguments were not provided, maybe use `#[schema(name = 'foo', version = 0.0)]?`")
        }
    }
}

fn get_properties(
    input: &DataStruct,
) -> Result<(Vec<PropertyInput>, Vec<TriggerInput>), proc_macro::TokenStream> {
    let mut to_ret_props = vec![];
    let mut to_ret_triggers = vec![];
    match &input.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                if check_field(field)? {
                    match PropertyInput::from_field(field) {
                        Err(e) => {
                            abort!(e.span(), e; help = "It seems there is a field which does not have the `#[field] macro applied to, perhahps try using it?`")
                        }
                        Ok(e) => to_ret_props.push(e),
                    };
                } else {
                    match TriggerInput::from_field(field) {
                        Err(e) => {
                            abort!(e.span(), e; help = "It seems a error occured while trying to parse the trigger input")
                        }
                        Ok(e) => to_ret_triggers.push(e),
                    }
                }
            }
        }
        _ => {
            abort!(Span::call_site(), "Fields found in struct that are not named, which is not supported!"; help = "Try naming your fields!")
        }
    }
    return Ok((to_ret_props, to_ret_triggers));
}

fn generate_for_trigger(
    TriggerInput { name, ty, ident }: TriggerInput,
) -> proc_macro2::TokenStream {
    let name = name.unwrap_or(ident.to_token_stream().to_string());
    let sig = quote::quote!(<#ty as ::gsd_rs::zvariant::Type>::signature());

    return quote::quote!(
        ::gsd_rs::trigger::Trigger::new(#name.to_string(), #sig)
    );
}

fn generate_for_property(property: PropertyInput) -> proc_macro2::TokenStream {
    let name = property
        .name
        .unwrap_or(property.ident.to_token_stream().to_string());

    let about = property.about.unwrap_or("".to_string());
    let long_about = property.long_about.unwrap_or("".to_string());
    let show_in_settings = property.show_in_settings.unwrap_or(true);
    let writable = property.writable.unwrap_or(true);
    let sig = property.ty;

    let default = property.default.map(|ident |{
        quote::quote!(
            .default(::gsd_rs::value::Value::new(#ident (), <#sig as ::gsd_rs::zvariant::Type>::signature())?)
        )
    }).unwrap_or_default();

    let value = property.value.map(|ident| {
        quote::quote!(
            .value(::gsd_rs::value::Value::new(#ident (), <#sig as ::gsd_rs::zvariant::Type>::signature())?)
        )
    }).unwrap_or_default();

    let choices = property
        .choices
        .map(|ident| {
            quote::quote!(
                .choices_sig(<#sig as ::gsd_rs::zvariant::Type>::signature(), #ident ())
            )
        })
        .unwrap_or_default();

    let stream = quote::quote!(
        ::gsd_rs::property::Property::builder()
            .name(#name.to_string())
            .about(#about.to_string())
            .long_about(#long_about.to_string())
            .show_in_settings(#show_in_settings)
            .writable(#writable)
            .signature(<#sig as ::gsd_rs::zvariant::Type>::signature())
            #default
            #value
            #choices
            .build()?
    );
    stream
}

fn check_field(field: &Field) -> Result<bool, proc_macro::TokenStream> {
    let mut is_field: bool = false;
    let mut is_trigger: bool = false;

    field.attrs.iter().for_each(|attr| {
        if attr.path().is_ident("field") {
            is_field = true;
        }
        if attr.path().is_ident("trigger") {
            is_trigger = true;
        }
    });

    if is_field && is_trigger {
        abort!(Span::call_site(), "a field cannot be both a trigger and a property"; help = "Try creating a seperate field for a trigger/property")
    } else if is_trigger {
        return Ok(false);
    } else {
        return Ok(true);
    }
}

fn generate_schema<'a>(
    properties: impl Iterator<Item = TokenStream> + 'a,
    triggers: impl Iterator<Item = TokenStream> + 'a,
    schema_input: SchemaInput,
) -> TokenStream {
    let name = schema_input.name;
    let version = schema_input.version;
    let stream = quote::quote!(
        Ok(::gsd_rs::schema::Schema::builder()
            .name(#name.to_string())
            .version(#version)
            .properties(::std::vec![#(#properties),*])
            .triggers(::std::vec![#(#triggers),*])
            .build()?)
    );
    stream
}
