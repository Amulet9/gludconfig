use darling::{FromAttributes, FromMeta};
use proc_macro2::Span;
use proc_macro_error::{abort, OptionExt};
use quote::{format_ident, ToTokens};
use syn::{
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Ge, RArrow},
    Attribute, ItemTrait, PatTuple, PatTupleStruct, Path, ReturnType, TraitItemFn, Visibility,
};

#[derive(FromMeta, Debug)]
pub struct GenCodeInput {
    name: String,
    blocking: bool,
}

#[derive(FromAttributes, Debug)]
#[darling(attributes(property, trigger))]
pub struct Input {
    #[darling(default)]
    name: Option<String>,
}

#[derive(Debug)]
enum FunctionType {
    Property,
    Trigger,
}

pub fn expand(
    input: proc_macro::TokenStream,
    args: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _trait = parse_macro_input!(args as ItemTrait);
    let args = match parse_args(input) {
        Ok(e) => e,
        Err(v) => return v.into(),
    };

    let _functions = match _trait
        .items
        .into_iter()
        .map(|item| match item {
            syn::TraitItem::Fn(_fn) => Some(_fn),
            _ => None,
        })
        .flatten()
        .map(|item| {
            generate_for_function(item, &_trait.ident, &args.name, &_trait.vis, args.blocking)
        })
        .collect::<Result<Vec<_>, proc_macro2::TokenStream>>()
    {
        Ok(_fns) => _fns,
        Err(e) => return e.into(),
    };

    let schema = match generate_schema(args.name, &_trait.ident, &_trait.vis, args.blocking) {
        Ok(e) => e,
        Err(e) => return e.into(),
    };

    quote::quote!( #schema #(#_functions)* ).into()
}

pub fn generate_schema(
    name: String,
    ident: &syn::Ident,
    vis: &syn::Visibility,
    blocking: bool,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let ty = blocking
        .then(|| quote::quote!(::zbus::blocking::Proxy))
        .unwrap_or(quote::quote!(::zbus::Proxy));

    let new_fn = blocking.then(|| quote::quote!(
        fn new(conn: &zbus::blocking::Connection) -> ::zbus::Result<#ident> {
            Ok(Self {
                proxy: #ty::new(conn, "org.glud.GludConfig", "/org/glud/gludconfig", "org.glud.GludConfig")?
            })
        }
    )).unwrap_or(quote::quote!(
        async fn new(conn: &zbus::Connection) -> ::zbus::Result<#ident> {
            Ok(Self {
                proxy: #ty::new(conn, "org.glud.GludConfig", "/org/glud/gludconfig", "org.glud.GludConfig").await?
            })
        }
    ));

    let stream = quote::quote!(
        #vis struct #ident {
            proxy: #ty<'static>
        }

        impl #ident {
            #new_fn
        }
    );
    Ok(stream)
}

pub fn generate_for_function(
    mut _fn: TraitItemFn,
    schema_ident: &syn::Ident,
    schema_name: &String,
    vis: &syn::Visibility,
    blocking: bool,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    if _fn.sig.unsafety.is_some() {
        abort!(Span::call_site(), "Functions passed to gen code cannot be unsafe"; help = "Remove the unsafe code")
    }

    if _fn.sig.asyncness.is_none() && !blocking {
        abort!(Span::call_site(), "Non blocking interfaces must always be async"; help = "If you wish to use blocking code, set blocking to true.")
    }

    let ty = check_attrs_for_function(&_fn.attrs)?;

    let input = match Input::from_attributes(&_fn.attrs) {
        Ok(e) => e,
        Err(r) => return Err(r.write_errors()),
    };
    let ident = &_fn.sig.ident;
    let name = input.name.unwrap_or(ident.to_string());
    let generics = &_fn.sig.generics;
    let target_ty = match &_fn.sig.output {
        ReturnType::Default => quote::quote!(()),
        ReturnType::Type(_, ty) => quote::quote!( #ty ),
    };

    let (_async, _await) = blocking
        .then(|| (Default::default(), Default::default()))
        .unwrap_or((quote::quote!(async), quote::quote!(.await)));

    let stream = match ty {
        FunctionType::Trigger => {
            let emit_trigger_ident = format_ident!("{}", ident);
            let listen_trigger_ident = format_ident!("{}_occured", ident);
            let change_struct_event = format_ident!("{}ChangeStream", ident);

            let trigger_occur_ty = blocking
                .then(|| quote::quote!(::zbus::blocking::SignalIterator<'static>))
                .unwrap_or(quote::quote!(::zbus::SignalStream<'static>));

            let stream = quote::quote!(
                impl #schema_ident {
                    pub #_async fn #emit_trigger_ident #generics(&self, value: #target_ty) -> ::zbus::Result<()> {
                        let value = ::zbus::zvariant::Value::new(value).to_owned();
                        self.proxy.call::<_, _, ()>("Trigger", &(#schema_name, #name, value))#_await
                    }

                    pub #_async fn #listen_trigger_ident #generics(&self) -> ::zbus::Result<#trigger_occur_ty> {
                        self.proxy.receive_signal("TriggerInvoked")#_await
                    }
                }
            );

            stream
        }
        FunctionType::Property => {
            let get_ident = format_ident!("{}", ident);
            let set_ident = format_ident!("set_{}", ident);
            let reset_ident = format_ident!("reset_{}", ident);
            let info_ident = format_ident!("info_{}", ident);
            let change_ident = format_ident!("{}_changed", ident);

            let change_ty = blocking
                .then(|| quote::quote!(::zbus::blocking::SignalIterator<'static>))
                .unwrap_or(quote::quote!(::zbus::SignalStream<'static>));

            let stream = quote::quote!(
                impl #schema_ident {
                    pub #_async fn #info_ident #generics(&self) -> ::zbus::Result<(bool, String, String, bool, ::zbus::zvariant::OwnedSignature)> {
                        self.proxy.call::<_, _, (bool, String, String, bool, ::zbus::zvariant::OwnedSignature)>("Info", &(#schema_name, #name))#_await
                    }

                    pub #_async fn #change_ident #generics(&self) -> ::zbus::Result<#change_ty> {
                        self.proxy.receive_signal_with_args("PropertyChanged", &[(0, #schema_name), (1, #name)])#_await
                    }

                    pub #_async fn #get_ident #generics(&self) -> ::zbus::Result<Option<#target_ty>> {
                        let (is_null, value) = self.proxy.call::<_, _, (bool, ::zbus::zvariant::OwnedValue)>("Read", &(#schema_name, #name))#_await?;
                        if is_null {
                            return Ok(None)
                        } else {
                            let value = match ::std::convert::Into::<::zbus::zvariant::Value<'static>>::into(value) {
                               ::zbus::zvariant::Value::Value(v) => <#target_ty>::try_from(*v).ok(),
                                other => <#target_ty>::try_from(other).ok(),
                            };
                            return Ok(value)
                        }
                    }
                    pub #_async fn #set_ident #generics(&self, value: ::core::option::Option<#target_ty>) -> ::zbus::Result<()> {
                        let (is_null, value) = match value {
                            None => (true, ::zbus::zvariant::Value::from(true).to_owned()),
                            Some(value) => (false, ::zbus::zvariant::Value::from(value).to_owned()),
                        };
                        Ok(self.proxy.call::<_, _, ()>("Set", &(#schema_name, #name, (is_null, value)))#_await?)
                    }
                    pub #_async fn #reset_ident #generics(&self) -> ::zbus::Result<bool> {
                        let value = self.proxy.call::<_, _, bool>("Reset", &(#schema_name, #name))#_await?;
                        Ok(value)
                    }

                }
            );

            stream
        }
    };

    return Ok(stream);
}

fn check_attrs_for_function(attrs: &[Attribute]) -> Result<FunctionType, proc_macro2::TokenStream> {
    let mut ty: &str = "uninit";
    let mut count = 0;

    attrs.iter().for_each(|attr| {
        let path = attr.path();
        if let Some(ident) = path.get_ident() {
            match &*ident.to_string() {
                "property" => {
                    ty = "property";
                    count += 1;
                }
                "trigger" => {
                    ty = "trigger";
                    count += 1;
                }
                _ => {}
            }
        }
    });

    if count > 1 {
        abort!(Span::call_site(), "It looks like the function is marked by multiple glud attributes, which is not allowed"; help = "Remove the attribute")
    };

    match ty {
        "property" => Ok(FunctionType::Property),
        "trigger" => Ok(FunctionType::Trigger),
        _ => {
            abort!(Span::call_site(), "Function is unmarked by any glud attribute"; help = "Please do so")
        }
    }
}

pub fn parse_args(args: proc_macro::TokenStream) -> Result<GenCodeInput, proc_macro2::TokenStream> {
    let meta = match darling::ast::NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => return Err(e.into_compile_error()),
    };

    match GenCodeInput::from_list(&meta) {
        Ok(e) => Ok(e),
        Err(e) => return Err(e.write_errors()),
    }
}
