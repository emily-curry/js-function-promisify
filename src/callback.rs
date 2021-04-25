use js_sys::Function;
use std::fmt::Debug;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub struct Callback<F: ?Sized> {
  closure: Closure<F>,
}

impl<F: ?Sized> Callback<F> {
  pub fn new(closure: Closure<F>) -> Self
  where
    F: 'static,
  {
    Self { closure }
  }

  /// TODO: Should this be "as_function" or "to_function"?
  pub fn as_function(&self) -> Function {
    let js_func: JsValue = self.closure.as_ref().into();
    let func: Function = js_func.into();
    func
  }

  pub fn as_closure(&self) -> &Closure<F> {
    &self.closure
  }
}

/// Marker trait for `Callback`s.
pub trait CallbackMarker {}

impl std::fmt::Debug for dyn CallbackMarker {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "CallbackMarker")
  }
}

impl CallbackMarker for Callback<dyn FnMut()> {}
impl CallbackMarker for Callback<dyn FnMut(JsValue)> {}
impl CallbackMarker for Callback<dyn FnMut(JsValue, JsValue)> {}
impl CallbackMarker for Callback<dyn FnMut(JsValue, JsValue, JsValue)> {}
impl CallbackMarker for Callback<dyn FnMut(JsValue, JsValue, JsValue, JsValue)> {}
impl CallbackMarker for Callback<dyn FnMut(JsValue, JsValue, JsValue, JsValue, JsValue)> {}
impl CallbackMarker for Callback<dyn FnMut(JsValue, JsValue, JsValue, JsValue, JsValue, JsValue)> {}
