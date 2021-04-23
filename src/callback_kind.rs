use wasm_bindgen::JsValue;

pub enum CallbackKind {
  Arg0(Box<dyn FnMut() -> Result<JsValue, JsValue>>),
  Arg1(Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>),
  Arg2(Box<dyn FnMut(JsValue, JsValue) -> Result<JsValue, JsValue>>),
  Arg3(Box<dyn FnMut(JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>>),
  Arg4(Box<dyn FnMut(JsValue, JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>>),
  Arg5(Box<dyn FnMut(JsValue, JsValue, JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>>),
}

impl std::fmt::Debug for CallbackKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      CallbackKind::Arg0(_) => write!(f, "CallbackKind::Arg0"),
      CallbackKind::Arg1(_) => write!(f, "CallbackKind::Arg1"),
      CallbackKind::Arg2(_) => write!(f, "CallbackKind::Arg2"),
      CallbackKind::Arg3(_) => write!(f, "CallbackKind::Arg3"),
      CallbackKind::Arg4(_) => write!(f, "CallbackKind::Arg4"),
      CallbackKind::Arg5(_) => write!(f, "CallbackKind::Arg5"),
    }
  }
}
