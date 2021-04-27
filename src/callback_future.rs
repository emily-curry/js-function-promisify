use crate::Callback;
use crate::CallbackMarker;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::task::Poll;
use std::task::Waker;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;

fn finish(state: &RefCell<CallbackFutureInner>, val: Result<JsValue, JsValue>) {
  let task = {
    let mut state = state.borrow_mut();
    debug_assert!(state.result.is_none());
    for cb in state.cb.to_owned().into_iter() {
      drop(cb);
    }
    state.result = Some(val);
    state.task.take()
  };

  if let Some(task) = task {
    task.wake()
  }
}

#[derive(Debug)]
pub struct CallbackFuture {
  inner: Rc<RefCell<CallbackFutureInner>>,
}

impl CallbackFuture {
  /// Creates a new `CallbackFuture` ... TODO:
  pub fn new() -> Self {
    Self {
      inner: CallbackFutureInner::new(),
    }
  }

  pub fn get_arg0<F>(&self, mut cb: F) -> Rc<Callback<dyn FnMut()>>
  where
    F: 'static + FnMut() -> Result<JsValue, JsValue>,
  {
    let state = Rc::clone(&self.inner);
    let closure = Closure::once(move || finish(&state, cb()));
    let callback = Callback::new(closure);
    self.register_callback(callback)
  }

  pub fn get_arg1<F>(&self, mut cb: F) -> Rc<Callback<dyn FnMut(JsValue)>>
  where
    F: 'static + FnMut(JsValue) -> Result<JsValue, JsValue>,
  {
    let state = Rc::clone(&self.inner);
    let closure = Closure::once(move |a1| finish(&state, cb(a1)));
    let callback = Callback::new(closure);
    self.register_callback(callback)
  }

  pub fn get_arg2<F>(&self, mut cb: F) -> Rc<Callback<dyn FnMut(JsValue, JsValue)>>
  where
    F: 'static + FnMut(JsValue, JsValue) -> Result<JsValue, JsValue>,
  {
    let state = Rc::clone(&self.inner);
    let closure = Closure::once(move |a1, a2| finish(&state, cb(a1, a2)));
    let callback = Callback::new(closure);
    self.register_callback(callback)
  }

  pub fn get_arg3<F>(&self, mut cb: F) -> Rc<Callback<dyn FnMut(JsValue, JsValue, JsValue)>>
  where
    F: 'static + FnMut(JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>,
  {
    let state = Rc::clone(&self.inner);
    let closure = Closure::once(move |a1, a2, a3| finish(&state, cb(a1, a2, a3)));
    let callback = Callback::new(closure);
    self.register_callback(callback)
  }

  pub fn get_arg4<F>(
    &self,
    mut cb: F,
  ) -> Rc<Callback<dyn FnMut(JsValue, JsValue, JsValue, JsValue)>>
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
  ) -> Rc<Callback<dyn FnMut(JsValue, JsValue, JsValue, JsValue, JsValue)>>
  where
    F: 'static + FnMut(JsValue, JsValue, JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>,
  {
    let state = Rc::clone(&self.inner);
    let closure = Closure::once(move |a1, a2, a3, a4, a5| finish(&state, cb(a1, a2, a3, a4, a5)));
    let callback = Callback::new(closure);
    self.register_callback(callback)
  }

  pub fn get_arg6<F>(
    &self,
    mut cb: F,
  ) -> Rc<Callback<dyn FnMut(JsValue, JsValue, JsValue, JsValue, JsValue, JsValue)>>
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

  pub fn get(&self) -> Rc<Callback<dyn FnMut(JsValue)>> {
    self.get_arg1(|a| Ok(a))
  }

  pub fn get_resolve(&self) -> Rc<Callback<dyn FnMut(JsValue)>> {
    self.get_arg1(|a| Ok(a))
  }

  pub fn get_reject(&self) -> Rc<Callback<dyn FnMut(JsValue)>> {
    self.get_arg1(|a| Err(a))
  }

  pub fn get_node(&self) -> Rc<Callback<dyn FnMut(JsValue, JsValue)>> {
    self.get_arg2(|err, data| {
      if err == JsValue::UNDEFINED || err == JsValue::NULL {
        return Ok(data);
      }
      Err(err)
    })
  }

  fn register_callback<F>(&self, cb: F) -> Rc<F>
  where
    F: 'static + CallbackMarker,
  {
    let ptr = Rc::new(cb);
    let ret = Rc::clone(&ptr);
    let mut state = self.inner.borrow_mut();
    state.cb.push(ptr);
    ret
  }
}

impl Future for CallbackFuture {
  type Output = Result<JsValue, JsValue>;

  fn poll(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Self::Output> {
    let mut inner = self.inner.borrow_mut();
    if let Some(val) = inner.result.take() {
      return Poll::Ready(val);
    }
    inner.task = Some(cx.waker().clone());
    Poll::Pending
  }
}

#[derive(Debug)]
struct CallbackFutureInner {
  cb: Vec<Rc<dyn CallbackMarker>>,
  result: Option<Result<JsValue, JsValue>>,
  task: Option<Waker>,
}

impl CallbackFutureInner {
  fn new() -> Rc<RefCell<CallbackFutureInner>> {
    Rc::new(RefCell::new(CallbackFutureInner {
      cb: Vec::new(),
      task: None,
      result: None,
    }))
  }
}
