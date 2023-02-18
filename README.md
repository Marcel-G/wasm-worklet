# waw-rs

`waw-rs` should help you create WebAudio Worklets using Rust, without crying.

I would recommend to read [WebAssembly/Rust Tutorial: Pitch-perfect Audio Processing](https://www.toptal.com/webassembly/webassembly-rust-tutorial-web-audio)

This is very all very experimental.

## Usage

To use waw-rs in your project, add it as a dependency in your Cargo.toml:

```toml
waw = { git = "https://github.com/Marcel-G/waw-rs" }
```

You will need to setup an xtask to build the project, see [xtask-waw](xtask-waw/README.md).

Then, in your Rust code, simply implement the `AudioModule` trait and call the `waw::main!` macro on your struct:

`src/lib.rs`

```rust
use waw::{
  worklet::{ AudioModule, Emitter },
  buffer::{ AudioBuffer, ParamBuffer }
};

struct MyWorklet;

impl AudioModule for MyWorklet {
  fn create(
    _initial_state: Option<Self::InitialState>,
    _emitter: Emitter<Self::Event>
  ) -> Self { MyWorklet }
  fn process(&mut self, audio: &mut AudioBuffer, params: &ParamBuffer<Self::Param>) {
    // Implement process
  }
}

waw::main!(MyWorklet);
```

Run the build using the xtask command

```bash
cargo xtask dist
```

They can then be used from JavaScript:

```typescript
import init, { init_worklet, Oscillator, Gain } from "./pkg/waw-demo";
import worklet_url from "./pkg/waw-demo.worklet.js?url&worker";
// Note: waw-demo.worklet.js must be loaded as a URL - bundlers may need different config for this

const main = async () => {
  const context = new AudioContext();

  // Init WASM on the main thread
  await init();
  // Init WASM on the audio worklet thread
  await init_worklet(context, worklet_url);

  // Create an audio context
  const context = new AudioContext();

  // Call create on the generated worklet node
  const worklet = await MyWorklet.create(context);

  // Connect the audio node to WebAudio graph
  worklet.node().connect(context.destination);

  // Wait for some interaction on the page before starting the audio
  const handle_interaction = () => {
    void context?.resume();
  };
  document.addEventListener("click", handle_interaction, { once: true });
};

main();
```

See the [demo project](demo/app) for a full example.

## Links

- [wasm-bindgen WASM audio worklet](https://rustwasm.github.io/wasm-bindgen/examples/wasm-audio-worklet.html#wasm-audio-worklet)
