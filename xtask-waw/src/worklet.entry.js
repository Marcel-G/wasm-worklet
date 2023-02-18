// import * as bindgen from "."; -- Import added at build time

function registerAll () {
  for (const name of Object.keys(bindgen).filter((name) => /_.+Processor/.test(name))) {
    registerProcessor(
      name,
      class _AudioWorklet extends AudioWorkletProcessor {
        static get parameterDescriptors() {
          return JSON.parse(bindgen[name].parameter_descriptor());
        }
        constructor(options) {
          super();
          this.options = options;
          const [initial_state] = options.processorOptions || [];
          this.processor = new bindgen[name](this, initial_state);
          this.port.postMessage({ method: "send_wasm_program_done" });
          this.processor.connect();
        }

        process(inputs, outputs, parameters) {
          if (
            this.processor &&
            !this.processor.process(inputs, outputs, parameters)
          ) {
            this.processor.free();
            return false;
          }
          return true;
        }
      }
    );
  }
}

registerProcessor(
  "_init",
  class __InitWorklet extends AudioWorkletProcessor {
    constructor(options) {
      super();
      this.options = options;
      const [module] = options.processorOptions || [];
      bindgen.initSync(module);
      registerAll()
      this.port.postMessage({ method: "send_wasm_program_done" });
    }

    process() { return false }
  }
)
