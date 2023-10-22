use proc_macro::TokenStream as PStream;
use proc_macro2::TokenStream as P2Stream;
use proc_macro_error::{abort, proc_macro_error};
use syn::parse_macro_input;

mod generate_code;
mod error;
mod schema;

#[proc_macro_derive(Schema, attributes(schema, field, trigger))]
#[proc_macro_error]
pub fn schema(input: PStream) -> PStream {
    schema::expand(input)
}


#[proc_macro_error]
#[proc_macro_attribute]
pub fn glud_interface(input: PStream, args: PStream) -> PStream {
    generate_code::expand(input, args)
}
