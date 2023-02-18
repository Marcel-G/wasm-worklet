use proc_macro2::{Ident as ProcIdent, Span};
use quote::quote;
use syn::Ident;

// Generates the node wrapper to bind with JS
pub fn node_wrapper(ident: &Ident) -> proc_macro2::TokenStream {
    let node_ident = ProcIdent::new(&format!("{ident}Node"), Span::call_site());

    quote! {
      #[wasm_bindgen(js_name = #ident)]
      pub struct #node_ident (waw::node::Node<#ident>);

      #[wasm_bindgen(js_class = #ident)]
      impl #node_ident {
          pub async fn create(
            ctx: waw::web_sys::AudioContext,
            initial_state: Option<<#ident as waw::worklet::AudioModule>::InitialState>
          ) -> Result<#node_ident, wasm_bindgen::JsValue> {
              let result = waw::node::Node::<#ident>::create(ctx, initial_state).await?;
              Ok(#node_ident(result))
          }
          pub fn node(&self) -> Result<waw::web_sys::AudioWorkletNode, wasm_bindgen::JsValue> {
            Ok(self.0.inner.clone())
          }

          pub fn get_param(&self, param: <#ident as waw::worklet::AudioModule>::Param) -> waw::web_sys::AudioParam {
            self.0.get_param(param)
          }

          pub fn command(&self, message: <#ident as waw::worklet::AudioModule>::Command) {
              self.0.command(message)
          }

          pub fn subscribe(&mut self, callback: waw::utils::callback::Callback<<#ident as waw::worklet::AudioModule>::Event>) {
              self.0.subscribe(callback.0)
          }

          pub fn destroy(&mut self) {
            self.0.destroy();
          }
      }
    }
}
