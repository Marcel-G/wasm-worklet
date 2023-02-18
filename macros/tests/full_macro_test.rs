use tsify::Tsify;
use waw::{
    buffer::{AudioBuffer, ParamBuffer},
    enum_map::Enum,
    serde::{Deserialize, Serialize},
    worklet::{AudioModule, Emitter},
};
use waw_macros::{ParameterDescriptor, RawHackDescribe};

#[test]
fn full_macro_test() {
    #[derive(Serialize, Deserialize, Tsify, Clone, RawHackDescribe)]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    #[serde(crate = "waw::serde")]
    pub enum TestEvent {
        One(bool),
        Two,
    }

    impl From<JsValue> for TestEvent {
        fn from(value: JsValue) -> Self {
            Self::from_js(value).unwrap()
        }
    }

    impl From<TestEvent> for JsValue {
        fn from(val: TestEvent) -> Self {
            val.into_js().unwrap().into()
        }
    }

    #[derive(Serialize, Deserialize, Tsify, Clone)]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    #[serde(crate = "waw::serde")]
    pub enum TestCommand {
        Three(Vec<f32>),
        Four,
    }

    impl From<JsValue> for TestCommand {
        fn from(value: JsValue) -> Self {
            Self::from_js(value).unwrap()
        }
    }

    impl From<TestCommand> for JsValue {
        fn from(val: TestCommand) -> Self {
            val.into_js().unwrap().into()
        }
    }

    #[derive(Serialize, Deserialize, Tsify, Clone)]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    #[serde(crate = "waw::serde")]
    pub struct TestInitialState {
        count: u32,
    }

    impl From<JsValue> for TestInitialState {
        fn from(value: JsValue) -> Self {
            Self::from_js(value).unwrap()
        }
    }

    impl From<TestInitialState> for JsValue {
        fn from(val: TestInitialState) -> Self {
            val.into_js().unwrap().into()
        }
    }

    #[derive(ParameterDescriptor, Serialize, Deserialize, Enum, Debug, Tsify)]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    #[serde(crate = "waw::serde")]
    pub enum TestParam {
        #[param(automation_rate = "a-rate", max_value = 1.0, default_value = 2.0)]
        Test,
    }

    pub struct TestWorklet {
        emitter: Emitter<TestEvent>,
    }

    impl AudioModule for TestWorklet {
        type Event = TestEvent;
        type Command = TestCommand;
        type Param = TestParam;

        fn create(_initial_state: Option<Self::InitialState>, emitter: Emitter<TestEvent>) -> Self {
            TestWorklet { emitter }
        }

        fn on_command(&mut self, command: Self::Command) {
            match command {
                TestCommand::Three(_) => self.emitter.send(TestEvent::One(false)),
                TestCommand::Four => self.emitter.send(TestEvent::Two),
            }
        }

        fn process(&mut self, _audio: &mut AudioBuffer, _params: &ParamBuffer<Self::Param>) {
            todo!()
        }
    }

    waw_macros::main!(TestWorklet);
}
