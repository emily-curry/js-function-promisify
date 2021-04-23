use wasm_bindgen::JsValue;

pub enum CallbackKind {
  Arg0(Box<dyn FnMut() -> Result<JsValue, JsValue>>),
  Arg1(Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>),
  Arg2(Box<dyn FnMut(JsValue, JsValue) -> Result<JsValue, JsValue>>),
  Arg3(Box<dyn FnMut(JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>>),
  Arg4(Box<dyn FnMut(JsValue, JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>>),
  Arg5(Box<dyn FnMut(JsValue, JsValue, JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>>),
}
