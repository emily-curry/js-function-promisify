use crate::Callback;
use crate::CallbackMarker;
use std::any::Any;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::task::Poll;
use std::task::Waker;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;

impl CallbackFuture {
  pub fn get_arg2<F>(&self, mut cb: F) -> &Callback<dyn FnMut(JsValue, JsValue)>
  where
    F: 'static + FnMut(JsValue, JsValue) -> Result<JsValue, JsValue>,
  {
    let state = Rc::clone(&self.inner);
    let closure = Closure::once(move |a1, a2| finish(&state, cb(a1, a2)));
    let callback = Callback::new(closure);
    self.register_callback(callback)
  }

  pub fn get_arg3<F>(&self, mut cb: F) -> &Callback<dyn FnMut(JsValue, JsValue, JsValue)>
  where
    F: 'static + FnMut(JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>,
  {
    let state = Rc::clone(&self.inner);
    let closure = Closure::once(move |a1, a2, a3| finish(&state, cb(a1, a2, a3)));
    let callback = Callback::new(closure);
    self.register_callback(callback)
  }

  pub fn get_arg4<F>(&self, mut cb: F) -> &Callback<dyn FnMut(JsValue, JsValue, JsValue, JsValue)>
  where
    F: 'static + FnMut(JsValue, JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>,
  {
    let state = Rc::clone(&self.inner);
    let closure = Closure::once(move |a1, a2, a3, a4| finish(&state, cb(a1, a2, a3, a4)));
    let callback = Callback::new(closure);
    self.register_callback(callback)
  }

  pub fn get_arg5<F>(
    &self,
    mut cb: F,
  ) -> &Callback<dyn FnMut(JsValue, JsValue, JsValue, JsValue, JsValue)>
  where
    F: 'static + FnMut(JsValue, JsValue, JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>,
  {
    let state = Rc::clone(&self.inner);
    let closure = Closure::once(move |a1, a2, a3, a4, a5| finish(&state, cb(a1, a2, a3, a4, a5)));
    let callback = Callback::new(closure);
    let mut boxed = Box::new(callback);
    let ret = boxed.as_ref();
    self.inner.borrow_mut().cb.push(boxed);
    ret
  }

  pub fn get_arg6<F>(
    &self,
    mut cb: F,
  ) -> &Callback<dyn FnMut(JsValue, JsValue, JsValue, JsValue, JsValue, JsValue)>
  where
    F: 'static
      + FnMut(JsValue, JsValue, JsValue, JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>,
  {
    let state = Rc::clone(&self.inner);
    let closure =
      Closure::once(move |a1, a2, a3, a4, a5, a6| finish(&state, cb(a1, a2, a3, a4, a5, a6)));
    let callback = Callback::new(closure);
    self.register_callback(callback)
  }

  pub fn get(&self) -> &Callback<dyn FnMut(JsValue)> {
    self.get_arg1(|a| Ok(a))
  }

  pub fn get_resolve(&self) -> &Callback<dyn FnMut(JsValue)> {
    self.get_arg1(|a| Ok(a))
  }

  pub fn get_reject(&self) -> &Callback<dyn FnMut(JsValue)> {
    self.get_arg1(|a| Err(a))
  }

  pub fn get_node(&self) -> &Callback<dyn FnMut(JsValue, JsValue)> {
    self.get_arg2(|err, data| {
      if err == JsValue::UNDEFINED || err == JsValue::NULL {
        return Ok(data);
      }
      Err(err)
    })
  }
}
