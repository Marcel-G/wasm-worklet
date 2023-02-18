use std::{
    fmt::Debug,
    iter::Zip,
    ops::{Deref, DerefMut},
    slice::{Iter, IterMut},
};

use enum_map::{EnumArray, EnumMap};
use js_sys::{Array, Float32Array, Reflect};
use wasm_bindgen::{JsCast, JsValue};

/// Base audio buffer
#[derive(Clone)]
pub struct Buffer<const N: usize>([f32; N]);

impl<const N: usize> Deref for Buffer<N> {
    type Target = [f32; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for Buffer<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> Default for Buffer<N> {
    fn default() -> Self {
        Self([0.0; N])
    }
}

/// Multi-channel audio buffer
#[derive(Clone)]
pub struct ChannelBuffer<const N: usize> {
    channels: Vec<Buffer<N>>,
    /// When nothing is connected to this input / output
    is_connected: bool,
}

impl<const N: usize> ChannelBuffer<N> {
    /// Create an empty buffer with n_channels
    pub fn new(n_channels: usize) -> Self {
        let channels = vec![Default::default(); n_channels];
        ChannelBuffer::<N> {
            channels,
            is_connected: true,
        }
    }

    /// Returns the number of channels in the buffer
    pub fn channels(&self) -> usize {
        self.channels.len()
    }

    /// Returns an iterator over all channels.
    pub fn channel_iter(&self) -> Iter<Buffer<N>> {
        if self.is_connected {
            self.channels.iter()
        } else {
            [].iter()
        }
    }

    /// Returns an iterator that allows modifying each channel.
    pub fn channel_iter_mut(&mut self) -> IterMut<Buffer<N>> {
        if self.is_connected {
            self.channels.iter_mut()
        } else {
            [].iter_mut()
        }
    }

    /// Returns a reference to channel data or None if the index is out of bounds.
    pub fn channel(&self, index: usize) -> Option<&Buffer<N>> {
        if self.is_connected {
            self.channels.get(index)
        } else {
            None
        }
    }

    /// Returns a mutable reference to channel data or None if the index is out of bounds.
    pub fn channel_mut(&mut self, index: usize) -> Option<&mut Buffer<N>> {
        if self.is_connected {
            self.channels.get_mut(index)
        } else {
            None
        }
    }

    /// Copy the contents of this JS typed array into the destination Rust slice.
    /// This function will efficiently copy the memory from a typed array into this buffer.
    #[inline]
    pub fn copy_from(&mut self, input_js: &Array) {
        // Empty all the buffers
        self.channels.iter_mut().for_each(|b| b.fill(0.0));

        if input_js.length() == 0 {
            self.is_connected = false;
        } else {
            self.is_connected = true;

            input_js
                .iter()
                .zip(self.channels.iter_mut())
                .for_each(|(channel_js, channels_rs)| {
                    let channel = channel_js.unchecked_into::<Float32Array>();
                    channel.copy_to(channels_rs.as_mut())
                })
        }
    }

    /// Copy the contents of the source Rust slice into this JS typed array.
    /// This function will efficiently copy the memory from within the buffer to this typed array.
    #[inline]
    pub fn copy_to(&mut self, input_js: &Array) {
        input_js
            .iter()
            .zip(self.channels.iter())
            .for_each(|(channel_js, channels_rs)| {
                let channel = channel_js.unchecked_into::<Float32Array>();
                channel.copy_from(channels_rs.as_ref());
            });

        // Empty all the buffers
        self.channels.iter_mut().for_each(|b| b.fill(0.0));
    }
}

/// Audio buffer for an audio input (128 samples in length)
pub type Input = ChannelBuffer<128>;
/// Audio buffer for an audio output (128 samples in length)
pub type Output = ChannelBuffer<128>;

/// Owns input and output audio data.
pub struct AudioBuffer {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}

impl AudioBuffer {
    /// Construct a multi-input/output, multi-channel audio buffer.
    pub fn new(
        in_inputs: usize,
        in_channels: usize,
        out_inputs: usize,
        out_channels: usize,
    ) -> Self {
        let inputs = vec![Input::new(in_channels); in_inputs];
        let outputs = vec![Output::new(out_channels); out_inputs];
        AudioBuffer { inputs, outputs }
    }

    /// Copy AudioWorkletProcessor input data into buffer.
    ///
    /// inputs_js: An array of inputs connected to the node, each item of which is, in turn, an array of channels.
    /// Each channel is a Float32Array containing 128 samples.
    /// For example, inputs[n][m][i] will access n-th input, m-th channel of that input, and i-th sample of that channel.
    pub fn copy_from_input(&mut self, inputs_js: &Array) {
        inputs_js
            .iter()
            .zip(self.inputs.iter_mut())
            .for_each(|(input_js, input_rs)| {
                input_rs.copy_from(&input_js.unchecked_into::<Array>())
            })
    }

    /// Copy buffer output data to AudioWorkletProcessor output.
    ///
    /// outputs_js: An array of outputs that is similar to the inputs parameter in structure.
    /// It is intended to be filled during the execution of the process() method.
    /// Each of the output channels is filled with zeros by default — the processor will output silence unless the output arrays are modified.
    pub fn copy_to_output(&mut self, outputs_js: &Array) {
        outputs_js
            .iter()
            .zip(self.outputs.iter_mut())
            .for_each(|(output_js, output_rs)| {
                output_rs.copy_to(&output_js.unchecked_into::<Array>())
            })
    }

    /// Create an iterator over pairs of inputs and outputs.
    pub fn zip(&mut self) -> Zip<Iter<Input>, IterMut<Output>> {
        self.inputs.iter().zip(self.outputs.iter_mut())
    }

    /// Split this buffer into separate inputs and outputs.
    pub fn split(&mut self) -> (&[Input], &mut [Output]) {
        let inputs = &self.inputs;
        let outputs = &mut self.outputs;
        (inputs, outputs)
    }
}

/// Audio buffer for single parameter (128 samples in length)
pub type Param = Buffer<128>;

/// Owns audio buffers for all parameters.
pub struct ParamBuffer<P>(EnumMap<P, Param>)
where
    P: EnumArray<Param>;

impl<P> Default for ParamBuffer<P>
where
    P: EnumArray<Param>,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<P> ParamBuffer<P>
where
    P: EnumArray<Param> + Debug,
{
    /// Copy AudioWorkletProcessor param data into buffer.
    ///
    /// js_params: An object containing string keys and Float32Array values.
    /// For each custom AudioParam defined using the parameterDescriptors getter, the key in the object is a name of that AudioParam,
    /// and the value is a Float32Array. The values of the array are calculated by taking scheduled automation events into consideration.
    pub fn copy_from_params(&mut self, js_params: &JsValue) {
        for (param, buffer) in self.0.iter_mut() {
            if let Some(js_array) = Reflect::get(js_params, &format!("{param:?}").into()) // @todo -- Using debug for param names may be a bad idea.
                .ok()
                .filter(|val| !val.is_undefined()) // This case happens when using the Never type.
                .map(|val| val.unchecked_into::<Float32Array>())
            {
                match js_array.length() {
                    // If the automation rate of the parameter is "a-rate", the array will contain 128 values — one for each frame in the current audio block.
                    128 => {
                        js_array.copy_to(buffer.as_mut());
                    }

                    // If the automation rate is "k-rate", the array will contain a single value, which is to be used for each of 128 frames.
                    //
                    // If there's no automation happening during the time represented by the current block,
                    // the array may contain a single value that is constant for the entire block, instead of 128 identical values.
                    1 => {
                        buffer.fill(js_array.get_index(0));
                    }

                    // Other possibilities are not supported.
                    other => {
                        panic!("Float32Array length {other} not supported.");
                    }
                }
            }
        }
    }

    /// Get the buffer for a given parameter.
    pub fn get(&self, name: P) -> &Param {
        &self.0[name]
    }

    /// Create an iterator over all parameter buffers.
    pub fn iter(&self) -> enum_map::Iter<P, Param> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioBuffer, ParamBuffer};
    use enum_map::Enum;
    use js_sys::{Array, Float32Array, Object};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_js_copy_input() {
        let create_buffer = |_| Float32Array::new_with_length(128).fill(1.0, 0, 128);
        let create_channels = |_| Array::from_iter((0..2).map(create_buffer));

        let inputs = Array::from_iter((0..2).map(create_channels));

        let mut buffer = AudioBuffer::new(2, 2, 2, 2);

        buffer.copy_from_input(&inputs);

        for (input, _) in buffer.zip() {
            for channel in input.channel_iter() {
                assert_eq!(channel.iter().collect::<Vec<_>>(), vec![&1.0; 128]);
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_js_copy_output() {
        let create_buffer = |_| Float32Array::new_with_length(128).fill(0.0, 0, 128);
        let create_channels = |_| Array::from_iter((0..2).map(create_buffer));

        let outputs = Array::from_iter((0..2).map(create_channels));

        let mut buffer = AudioBuffer::new(2, 2, 2, 2);

        for (_, output) in buffer.zip() {
            for channel in output.channel_iter_mut() {
                for sample in channel.iter_mut() {
                    *sample = 1.0
                }
            }
        }

        buffer.copy_to_output(&outputs);

        for output in outputs.iter() {
            for channel in output.unchecked_into::<Array>().iter() {
                assert_eq!(
                    channel.unchecked_into::<Float32Array>().to_vec(),
                    vec![1.0; 128]
                )
            }
        }
    }

    #[derive(Debug, Enum)]
    enum TestParam {
        Volume,
    }

    #[wasm_bindgen_test]
    fn test_js_copy_a_rate_params() {
        let create_buffer = || Float32Array::new_with_length(128).fill(1.0, 0, 128);

        let params =
            Object::from_entries(&Array::of1(&Array::of2(&"Volume".into(), &create_buffer())))
                .unwrap();

        let mut buffer = ParamBuffer::<TestParam>::default();

        buffer.copy_from_params(&params);

        assert_eq!(buffer.get(TestParam::Volume).to_vec(), vec![1.0; 128])
    }

    #[wasm_bindgen_test]
    fn test_js_copy_k_rate_params() {
        let create_buffer = || Float32Array::new_with_length(1).fill(1.0, 0, 1);

        let params =
            Object::from_entries(&Array::of1(&Array::of2(&"Volume".into(), &create_buffer())))
                .unwrap();

        let mut buffer = ParamBuffer::<TestParam>::default();

        buffer.copy_from_params(&params);

        assert_eq!(buffer.get(TestParam::Volume).to_vec(), vec![1.0; 128])
    }
}
