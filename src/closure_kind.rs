use js_sys::Function;
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

impl std::fmt::Debug for ClosureKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      ClosureKind::Arg0(_) => write!(f, "ClosureKind::Arg0"),
      ClosureKind::Arg1(_) => write!(f, "ClosureKind::Arg1"),
      ClosureKind::Arg2(_) => write!(f, "ClosureKind::Arg2"),
      ClosureKind::Arg3(_) => write!(f, "ClosureKind::Arg3"),
      ClosureKind::Arg4(_) => write!(f, "ClosureKind::Arg4"),
      ClosureKind::Arg5(_) => write!(f, "ClosureKind::Arg5"),
    }
  }
}

impl ClosureKind {
  /// TODO: Should this be "as_function" or "to_function"?
  pub fn to_function(&self) -> Function {
    let js_func: JsValue = match self {
      ClosureKind::Arg0(c) => c.as_ref().into(),
      ClosureKind::Arg1(c) => c.as_ref().into(),
      ClosureKind::Arg2(c) => c.as_ref().into(),
      ClosureKind::Arg3(c) => c.as_ref().into(),
      ClosureKind::Arg4(c) => c.as_ref().into(),
      ClosureKind::Arg5(c) => c.as_ref().into(),
    };
    let func: Function = js_func.into();
    func
  }
}
