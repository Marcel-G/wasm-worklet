use waw::{
    buffer::{AudioBuffer, ParamBuffer},
    worklet::{AudioModule, Emitter},
};

pub struct Gain;

impl AudioModule for Gain {
    fn create(_initialState: Option<Self::InitialState>, _emitter: Emitter<Self::Event>) -> Self {
        Gain
    }

    fn process(&mut self, audio: &mut AudioBuffer, _params: &ParamBuffer<Self::Param>) {
        for (input, output) in audio.zip() {
            // For each input sample and output sample in buffer
            for (in_channel, out_channel) in input.channel_iter().zip(output.channel_iter_mut()) {
                for (in_sample, out_sample) in in_channel.iter().zip(out_channel.iter_mut()) {
                    *out_sample = in_sample * 0.5
                }
            }
        }
    }
}

waw::main!(Gain);
