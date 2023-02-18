import init, { init_worklet, Oscillator, Gain } from './pkg/waw-demo';
import worklet_url from './pkg/waw-demo.worklet.js?url&worker';

const main = async () => {
  const context = new AudioContext();

  // Init WASM on the main thread
  await init();
  // Init WASM on the audio worklet thread
  await init_worklet(context, worklet_url)

  // Wait for some interaction on the page before starting the audio
  const handle_interaction = () => {
    void context?.resume()
  }
  document.addEventListener('click', handle_interaction, { once: true })

  // Create audio worklet nodes with `install`
  const node_a = await Oscillator.create(context, {
    count: 999
  });
  const node_b = await Gain.create(context);
  const node_b2 = await Gain.create(context);

  // Connect node to output
  node_a.node().connect(node_b.node());
  node_b.node().connect(context.destination);
  node_b2.node().connect(context.destination);

  // Call commands on the nodes
  let count = 0;
  setInterval(() => {
    count++;
    node_a.command({ Count: count });
  }, 1000);

  // Subscribe to events on the nodes
  node_a.subscribe((event) => {
    console.log("Got some event", event)
  })

  // Get audio parameters
  const frequency = node_a.get_param('Frequency');

  // Connect frequency AudioParam to the slider
  const frequencyControl = document.querySelector<HTMLInputElement>("#frequency");

  if (!frequencyControl) throw new Error("Can't find frequency slider")

  frequencyControl.addEventListener(
    "input",
    () => {
      frequency.value = parseInt(frequencyControl.value, 10)
    },
    false
  );
};

main();