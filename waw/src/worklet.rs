use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use enum_map::EnumArray;
use js_sys::{global, Array, Reflect};
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, AudioWorkletProcessor, MessageEvent,
    MessagePort,
};

use crate::buffer::AudioBuffer;
use crate::node::wait_for_any_response;
use crate::utils::environment::assert_main;
use crate::{
    buffer::{Param, ParamBuffer},
    types::{InternalMessage, Never, ParameterDescriptor, WorkletOptions},
    utils::environment::assert_worklet,
};

/// Used to communicate from the audio thread to the main thread
#[derive(Clone)]
pub struct Emitter<E> {
    port: MessagePort,
    _phantom: PhantomData<E>,
}

impl<E: Into<JsValue>> Emitter<E> {
    /// Construct a new emitter
    pub fn new(port: MessagePort) -> Self {
        Self {
            port,
            _phantom: PhantomData,
        }
    }

    /// Sends a message to the main thread
    pub fn send(&self, event: E) {
        self.port.post_message(&event.into()).ok();
    }
}

/// Audio worklet processor interface.
pub trait AudioModule {
    /// The type of messages sent from the audio worklet processor (audio thread) to the audio node (main thread).
    type Event: From<JsValue> + Into<JsValue> + FromWasmAbi = Never;

    /// The type of messages sent from the audio node (main thread) to audio worklet processor (audio thread).
    ///
    /// Commands are first converted from JS to WASM over ABI (main thread).
    /// Then, to JsValue for transmission via postMessage and
    /// then finally from JsValue in the worklet.
    type Command: From<JsValue> + Into<JsValue> + FromWasmAbi = Never;

    /// The type of parameters used by the worklet.
    type Param: EnumArray<Param> + ParameterDescriptor + Debug + FromWasmAbi = Never;

    /// State to initialise the module into
    type InitialState: From<JsValue> + Into<JsValue> + FromWasmAbi = Never;

    /// Number of inputs expected by the worklet.
    const INPUTS: u32 = 1;

    /// Number of outputs expected by the worklet.
    const OUTPUTS: u32 = 1;

    /// Constructor method for the worklet.
    fn create(initial_state: Option<Self::InitialState>, emitter: Emitter<Self::Event>) -> Self;

    /// Handler for commands from the audio node (main thread).
    fn on_command(&mut self, _command: Self::Command) {}

    /// Implements the audio processing algorithm for the audio processor worklet.
    fn process(&mut self, audio: &mut AudioBuffer, params: &ParamBuffer<Self::Param>);
}

/// Returns a float that represents the sample rate of the associated BaseAudioContext.
pub fn sample_rate() -> f64 {
    Reflect::get(&global(), &"sampleRate".into())
        .unwrap()
        .as_f64()
        .unwrap()
}

/// Returns a double that represents the ever-increasing context time of the audio block being processed.
/// It is equal to the currentTime property of the BaseAudioContext the worklet belongs to.
pub fn current_time() -> f64 {
    Reflect::get(&global(), &"currentTime".into())
        .unwrap()
        .as_f64()
        .unwrap()
}

/// Returns an integer that represents the ever-increasing current sample-frame of the audio block being processed.
/// It is incremented by 128 (the size of a render quantum) after the processing of each audio block.
pub fn current_frame() -> usize {
    Reflect::get(&global(), &"currentFrame".into())
        .unwrap()
        .as_f64()
        .unwrap() as usize
}

/// Wrapper struct for the audio worklet processor.
///
/// This struct is automatically generated when using the `waw::main!` macro. It should not be used directly.
#[doc(hidden)]
pub struct Processor<M: AudioModule> {
    rs_processor: Arc<Mutex<M>>,
    js_processor: AudioWorkletProcessor,
    enabled: Arc<AtomicBool>,
    audio: AudioBuffer,
    params: ParamBuffer<M::Param>,
    message_callback: Option<Closure<dyn Fn(MessageEvent)>>,
}

impl<M: AudioModule + 'static> Processor<M> {
    pub fn new(rs_processor: M, js_processor: AudioWorkletProcessor) -> Self {
        assert_worklet();
        // Use the js options to to allocate the buffers
        // `options` is non-standard, it's manually attached to `AudioWorkletProcessor` in the constructor.
        let js_options: AudioWorkletNodeOptions = Reflect::get(&js_processor, &"options".into())
            .expect("Can't find options on AudioWorkletProcessor")
            .unchecked_into();

        let options = WorkletOptions::from(js_options);

        let audio = AudioBuffer::new(
            options.number_of_inputs.try_into().unwrap(),
            options.channel_count.try_into().unwrap(),
            options.number_of_outputs.try_into().unwrap(),
            options.output_channel_count.try_into().unwrap(),
        );

        Processor {
            rs_processor: Arc::new(Mutex::new(rs_processor)),
            js_processor,
            enabled: Arc::new(AtomicBool::new(true)),
            audio,
            params: Default::default(),
            message_callback: None,
        }
    }

    /// Initialise bi-directional messaging between node and worklet.
    pub fn connect(&mut self) {
        // Add handler for inbound commands to module.
        let rs_processor = self.rs_processor.clone();
        let enabled = self.enabled.clone();
        let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Ok(internal_message) =
                // maybe convert this to a JS Symbol
                serde_wasm_bindgen::from_value::<InternalMessage>(event.data())
            {
                match internal_message {
                    InternalMessage::Destroy => {
                        enabled.store(false, Ordering::Relaxed);
                    }
                }
            } else {
                rs_processor.lock().unwrap().on_command(event.data().into());
            }
        }) as Box<dyn Fn(MessageEvent)>);

        self.js_processor
            .port()
            .unwrap()
            .add_event_listener_with_callback("message", callback.as_ref().unchecked_ref())
            .unwrap();

        self.message_callback = Some(callback);

        self.js_processor.port().unwrap().start();
    }

    /// Wrapper to convert JS process args into Rust structs
    pub fn process(&mut self, input: &Array, output: &Array, params: &JsValue) -> bool {
        self.audio.copy_from_input(input);
        self.params.copy_from_params(params);

        self.rs_processor
            .lock()
            .unwrap()
            .process(&mut self.audio, &self.params);

        self.audio.copy_to_output(output);

        self.enabled.load(Ordering::Relaxed)
    }
}

/// Equivalent of bindgen `init` for the worklet.
///
/// Given the URL to the worklet bindgen js this will initialise WASM in the worklet scope.
#[wasm_bindgen]
pub async fn init_worklet(ctx: AudioContext, js_url: &str) -> Result<(), JsValue> {
    assert_main();
    nop();

    // Loads up the worklet bootstrapper -- see: xtask-waw/src/worklet.entry.js
    JsFuture::from(ctx.audio_worklet()?.add_module(js_url)?).await?;

    let mut options = AudioWorkletNodeOptions::new();

    options.processor_options(Some(&js_sys::Array::of1(
        &wasm_bindgen::module(),
        // May be possible to introduce shared memory here, but it requires some extra compile flags & headers
        // https://github.com/rustwasm/wasm-bindgen/tree/main/examples/wasm-audio-worklet
        // &wasm_bindgen::memory(),
    )));

    // Initialise the fake `_init` audio worklet - it just initialises WASM
    // and registers all the proper worklets internally
    let node = AudioWorkletNode::new_with_options(&ctx, "_init", &options)?;

    wait_for_any_response(node.port()?).await?;

    Ok(())
}

// TextEncoder and TextDecoder are not available in [AudioWorkletGlobalScope](https://searchfox.org/mozilla-central/source/dom/webidl/AudioWorkletGlobalScope.webidl),
// but there is a dirty workaround: install stub implementations of these classes in globalThis.
// https://github.com/rustwasm/wasm-bindgen/blob/main/examples/wasm-audio-worklet/src/polyfill.js
//
// In some cases (error logging, JSON::parse / JSON::stringify) a proper polyfill is necessary rather than just a stub.
#[wasm_bindgen(module = "/src/polyfill/text-encoder-decoder.js")]
extern "C" {
    fn nop();
}
