#![feature(associated_type_defaults)]
#![feature(stmt_expr_attributes)]
#![doc = include_str!("../../README.md")]
#![warn(missing_docs)]

/// Audio buffer structs
pub mod buffer;
/// Audio node bindings (main thread)
pub mod node;
#[doc(hidden)]
pub mod types;
#[doc(hidden)]
pub mod utils;
/// Audio worklet bindings (audio thread)
pub mod worklet;

// Re-export dependencies that are used in macros.
#[doc(hidden)]
pub use enum_map;
#[doc(hidden)]
pub use js_sys;
#[doc(hidden)]
pub use serde;
#[doc(hidden)]
pub use serde_json;
#[doc(hidden)]
pub use tsify;
#[doc(hidden)]
pub use web_sys;

#[doc(hidden)]
pub use waw_macros as derive;

/// Generates the JS bindings
///
/// Given a struct implementing `AudioModule`, the [`main!`] will enable wasm-bindgen to
/// generate the neccesary JS to connect the Rust worklet to the WebAudio API.
///
/// ```
/// use waw::{
///   worklet::{ AudioModule, Emitter },
///   buffer::{ AudioBuffer, ParamBuffer }
/// };
///
/// struct MyWorklet;
///
/// impl AudioModule for MyWorklet {
///   fn create(_initial_state: Option<Self::InitialState>, _emitter: Emitter<Self::Event>) -> Self { MyWorklet }
///   fn process(&mut self, audio: &mut AudioBuffer, params: &ParamBuffer<Self::Param>) {
///     // Implement process
///   }
/// }
/// waw::main!(MyWorklet);
/// ```
pub use waw_macros::main;
