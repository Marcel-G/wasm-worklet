use node::node_wrapper;
mod node;
mod parameter_descriptor;
mod raw_hack_describe;
mod worklet;

use proc_macro::TokenStream;

use proc_macro2::{Ident as ProcIdent, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};
use worklet::worklet_wrapper;

#[proc_macro_derive(ParameterDescriptor, attributes(param))]
pub fn derive_parameters(input: TokenStream) -> TokenStream {
    parameter_descriptor::derive(input)
}

#[proc_macro]
pub fn main(input: TokenStream) -> TokenStream {
    let ident = parse_macro_input!(input as Ident);

    let processor = worklet_wrapper(&ident);
    // Build the bound node struct
    let node = node_wrapper(&ident);

    quote! {
      #processor
      #node
    }
    .into()
}

#[proc_macro_derive(RawHackDescribe)]
pub fn derive_raw_describe(input: TokenStream) -> TokenStream {
    raw_hack_describe::derive(input)
}

/// Adds necessary implementations for event enums
#[proc_macro_attribute]
pub fn derive_event(_metadata: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    let derive = parse_macro_input!(input as DeriveInput);
    let ident = derive.ident.clone();
    let module = ProcIdent::new(&format!("_mod_{ident}"), Span::call_site());

    quote! {
        mod #module {
            use super::*;
            use waw::tsify as tsify;
            use waw::tsify::Tsify;

            #[derive(
                waw::serde::Serialize,
                waw::serde::Deserialize,
                waw::tsify::Tsify,
                waw::derive::RawHackDescribe,
            )]
            #[tsify(into_wasm_abi, from_wasm_abi)]
            #[serde(crate = "waw::serde")]
            #derive

            impl From<wasm_bindgen::JsValue> for #ident {
                fn from(value: wasm_bindgen::JsValue) -> Self {
                    Self::from_js(value).unwrap()
                }
            }

            impl From<#ident> for wasm_bindgen::JsValue {
                fn from(val: #ident) -> Self {
                    val.into_js().unwrap().into()
                }
            }
        }

        use #module::#ident;
    }
    .into()
}

/// Adds necessary implementations for command enums
#[proc_macro_attribute]
pub fn derive_command(_metadata: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    let derive = parse_macro_input!(input as DeriveInput);
    let ident = derive.ident.clone();
    let module = ProcIdent::new(&format!("_mod_{ident}"), Span::call_site());

    quote! {
        mod #module {
            use super::*;
            use waw::tsify as tsify;
            use waw::tsify::Tsify;

            #[derive(waw::serde::Serialize, waw::serde::Deserialize, waw::tsify::Tsify, Clone)]
            #[tsify(into_wasm_abi, from_wasm_abi)]
            #[serde(crate = "waw::serde")]
            #derive

            impl From<wasm_bindgen::JsValue> for #ident {
                fn from(value: wasm_bindgen::JsValue) -> Self {
                    Self::from_js(value).unwrap()
                }
            }

            impl From<#ident> for wasm_bindgen::JsValue {
                fn from(val: #ident) -> Self {
                    val.into_js().unwrap().into()
                }
            }
        }

        use #module::#ident;
    }
    .into()
}

/// Adds necessary implementations for initial state struct
#[proc_macro_attribute]
pub fn derive_initial_state(_metadata: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    let derive = parse_macro_input!(input as DeriveInput);
    let ident = derive.ident.clone();
    let module = ProcIdent::new(&format!("_mod_{ident}"), Span::call_site());

    quote! {
        mod #module {
            use super::*;
            use waw::tsify as tsify;
            use waw::tsify::Tsify;

            #[derive(waw::serde::Serialize, waw::serde::Deserialize, waw::tsify::Tsify, Clone)]
            #[tsify(into_wasm_abi, from_wasm_abi)]
            #[serde(crate = "waw::serde")]
            #derive

            impl From<wasm_bindgen::JsValue> for #ident {
                fn from(value: wasm_bindgen::JsValue) -> Self {
                    Self::from_js(value).unwrap()
                }
            }

            impl From<#ident> for wasm_bindgen::JsValue {
                fn from(val: #ident) -> Self {
                    val.into_js().unwrap().into()
                }
            }
        }

        use #module::#ident;
    }
    .into()
}

/// Adds necessary implementations for parameter enums
#[proc_macro_attribute]
pub fn derive_param(_metadata: TokenStream, input: TokenStream) -> proc_macro::TokenStream {
    let derive = parse_macro_input!(input as DeriveInput);
    let ident = derive.ident.clone();
    let module = ProcIdent::new(&format!("_mod_{ident}"), Span::call_site());

    quote! {
        mod #module {
            use super::*;
            use waw::tsify as tsify;
            use waw::tsify::Tsify;

            #[derive(
                waw::serde::Serialize,
                waw::serde::Deserialize,
                waw::tsify::Tsify,
                waw::enum_map::Enum,
                waw::derive::ParameterDescriptor,
                Debug,
            )]
            #[tsify(into_wasm_abi, from_wasm_abi)]
            #[serde(crate = "waw::serde")]
            #derive
        }
        use #module::#ident;
    }
    .into()
}
