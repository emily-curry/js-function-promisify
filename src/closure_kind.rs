use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;

pub enum ClosureKind {
  Arg0(Closure<dyn FnMut()>),
  Arg1(Closure<dyn FnMut(JsValue)>),
  Arg2(Closure<dyn FnMut(JsValue, JsValue)>),
  Arg3(Closure<dyn FnMut(JsValue, JsValue, JsValue)>),
  Arg4(Closure<dyn FnMut(JsValue, JsValue, JsValue, JsValue)>),
  Arg5(Closure<dyn FnMut(JsValue, JsValue, JsValue, JsValue, JsValue)>),
}
