#[allow(unused_extern_crates)]
extern crate self as enroute_macros;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr};
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream};
use syn::Error;


struct EventDataMacroArgs {
    event_type: Expr,
    channel_name: Expr,
}

impl EventDataMacroArgs {
    fn from_attributes(attrs: &[syn::Attribute]) -> Option<EventDataMacroArgs> {
        for attr in attrs.iter().filter(|a| a.path().is_ident("event_data")) {
            if let Ok(args) = attr.parse_args::<EventDataMacroArgs>() {
                return Some(args);
            }
        }

        None
    }
}

impl Parse for EventDataMacroArgs {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let mut event_type = None;
        let mut channel_name = None;

        let args = Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated(input)?;
    
        for arg in args {
            if let syn::Meta::NameValue(nv) = arg {
                if nv.path.is_ident("event_type") {
                    event_type = Some(nv.value);
                } else if nv.path.is_ident("channel_name") {
                    channel_name = Some(nv.value);
                }
            }
        }

        Ok(EventDataMacroArgs {
            event_type: event_type.ok_or_else(|| Error::new(input.span(), "Missing event_type argument"))?,
            channel_name: channel_name.ok_or_else(|| Error::new(input.span(), "Missing channel_name argument"))?,
        })
    }
}

#[proc_macro_derive(EventData, attributes(event_data))]
pub fn derive_event_data(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let args = EventDataMacroArgs::from_attributes(&input.attrs).expect("Missing event_data attribute");
    let event_type = args.event_type;
    let channel_name = args.channel_name;

    let expanded = quote! {
        impl enroute::EventData for #name {
            fn event_type() -> &'static str {
                #event_type
            }

            fn channel_name() -> &'static str {
                #channel_name
            }
        }
    };

    TokenStream::from(expanded)
}