use futures::channel::oneshot;
use js_sys::Reflect;
use std::{marker::PhantomData, mem::forget};

use web_sys::{
    AddEventListenerOptions, AudioContext, AudioParam, AudioWorkletNode, AudioWorkletNodeOptions,
    MessageEvent, MessagePort,
};

use crate::{
    types::{AudioModuleDescriptor, InternalMessage},
    utils::environment::assert_main,
    worklet::AudioModule,
};

use wasm_bindgen::{prelude::*, JsCast};

/// Wrapper struct for the audio worklet node.
///
/// This struct is automatically generated when using the [`main!`]. It should not be used directly.
pub struct Node<Module: AudioModule + AudioModuleDescriptor> {
    /// The inner AudioWorkletNode
    pub inner: AudioWorkletNode,
    _phantom: PhantomData<Module>,
}

impl<Module: AudioModule + AudioModuleDescriptor> Node<Module> {
    /// Initialises a new audio worklet.
    pub async fn create(
        ctx: AudioContext,
        initial_state: Option<Module::InitialState>,
    ) -> Result<Node<Module>, JsValue> {
        assert_main();

        let mut options = AudioWorkletNodeOptions::new();
        options
            .number_of_inputs(Module::INPUTS)
            .number_of_outputs(Module::OUTPUTS);

        if let Some(init) = initial_state {
            options.processor_options(Some(&js_sys::Array::of1(&init.into())));
        }

        // Initialise the worklet node
        let node = AudioWorkletNode::new_with_options(&ctx, Module::processor_name(), &options)?;

        // Wait for the worklet processor to start.
        wait_for_any_response(node.port()?).await?;

        Ok(Node {
            inner: node,
            _phantom: PhantomData,
        })
    }

    /// Returns a given AudioParam.
    pub fn get_param(&self, param: Module::Param) -> AudioParam {
        Reflect::get(
            &js_sys::Object::from_entries(&self.inner.parameters().unwrap()).unwrap(),
            &format!("{param:?}").into(),
        )
        .unwrap()
        .unchecked_into()
    }

    /// Sends a command to the audio worklet processor.
    pub fn command(&self, message: Module::Command) {
        self.inner
            .port()
            .unwrap()
            .post_message(&message.into())
            .unwrap();
    }

    /// Sets up a subscription to events emitted from the audio worklet processor.
    pub fn subscribe(&mut self, callback: js_sys::Function) {
        let outer_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            callback.call1(&JsValue::NULL, &event.data()).unwrap();
        }) as Box<dyn Fn(MessageEvent)>);

        self.inner
            .port()
            .unwrap()
            .add_event_listener_with_callback("message", outer_callback.as_ref().unchecked_ref())
            .unwrap();

        forget(outer_callback); // @todo -- remove_event_listener on drop
    }

    /// Destroys the audio worklet processor.
    pub fn destroy(&mut self) {
        // Send message into worklet to destroy it
        self.inner
            .port()
            .unwrap()
            .post_message(&serde_wasm_bindgen::to_value(&InternalMessage::Destroy).unwrap())
            .unwrap();
    }
}

pub(crate) async fn wait_for_any_response(port: MessagePort) -> Result<(), JsValue> {
    let (sender, receiver) = oneshot::channel();

    let callback = Closure::once(Box::new(move |_| {
        sender.send(()).unwrap();
    }) as Box<dyn FnOnce(MessageEvent)>);

    port.add_event_listener_with_callback_and_add_event_listener_options(
        "message",
        callback.as_ref().unchecked_ref(),
        AddEventListenerOptions::new().once(true),
    )?;
    port.start();

    receiver.await.map_err(|_| JsValue::from_str("Cancelled"))?;

    Ok(())
}
