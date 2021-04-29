use core::cell::RefCell;
use js_sys::Function;
use std::fmt::Debug;
use std::future::Future;
use std::rc::Rc;
use std::task::Poll;
use std::task::Waker;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;

/// A `Callback<F>` is a wrapper around a `wasm_bindgen::prelude::Closure<F>` which supports TODO:
#[derive(Debug)]
pub struct Callback<F: 'static + ?Sized> {
  inner: Rc<RefCell<CallbackInner<F>>>,
}

impl<F: 'static + ?Sized> Callback<F> {
  pub fn as_function(&self) -> Function {
    let js_func: JsValue = self
      .inner
      .borrow()
      .cb
      .as_ref()
      .unwrap()
      .as_ref()
      .as_ref()
      .into();
    let func: Function = js_func.into();
    func
  }

  pub fn as_closure(&self) -> Rc<Closure<F>> {
    Rc::clone(self.inner.borrow().cb.as_ref().unwrap())
  }
}

impl<F: 'static + ?Sized> Future for Callback<F> {
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

impl<F> From<F> for Callback<dyn FnMut()>
where
  F: 'static + FnMut() -> Result<JsValue, JsValue>,
{
  fn from(mut cb: F) -> Self {
    let inner = CallbackInner::new();
    let state = Rc::clone(&inner);
    let closure = Closure::once(move || CallbackInner::finish(&state, cb()));
    let ptr = Rc::new(closure);
    inner.borrow_mut().cb = Some(ptr);
    Callback { inner }
  }
}

impl<F> From<F> for Callback<dyn FnMut(JsValue)>
where
  F: 'static + FnMut(JsValue) -> Result<JsValue, JsValue>,
{
  fn from(mut cb: F) -> Self {
    let inner = CallbackInner::new();
    let state = Rc::clone(&inner);
    let closure = Closure::once(move |a1| CallbackInner::finish(&state, cb(a1)));
    let ptr = Rc::new(closure);
    inner.borrow_mut().cb = Some(ptr);
    Callback { inner }
  }
}

impl Callback<dyn FnMut()> {
  pub fn from_arg0<T>(cb: T) -> Callback<dyn FnMut()>
  where
    T: 'static + FnMut() -> Result<JsValue, JsValue>,
  {
    Callback::from(cb)
  }

  pub fn from_arg1<T>(cb: T) -> Callback<dyn FnMut(JsValue)>
  where
    T: 'static + FnMut(JsValue) -> Result<JsValue, JsValue>,
  {
    Callback::from(cb)
  }
}

#[derive(Debug)]
pub struct CallbackInner<F: 'static + ?Sized> {
  cb: Option<Rc<Closure<F>>>,
  result: Option<Result<JsValue, JsValue>>,
  task: Option<Waker>,
}

impl<F: 'static + ?Sized> CallbackInner<F> {
  pub fn new() -> Rc<RefCell<CallbackInner<F>>> {
    Rc::new(RefCell::new(CallbackInner {
      cb: None,
      task: None,
      result: None,
    }))
  }

  pub fn finish(state: &RefCell<CallbackInner<F>>, val: Result<JsValue, JsValue>) {
    let task = {
      let mut state = state.borrow_mut();
      debug_assert!(state.result.is_none());
      debug_assert!(state.cb.is_some());
      drop(state.cb.take());
      state.result = Some(val);
      state.task.take()
    };
    if let Some(task) = task {
      task.wake()
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::Callback;
  use std::rc::Rc;
  use wasm_bindgen::JsCast;
  use wasm_bindgen_test::*;
  use web_sys::{window, IdbOpenDbRequest};

  wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

  #[wasm_bindgen_test]
  async fn function_dropped_after_await() {
    let future = Callback::from_arg0(|| Ok("".into()));
    let req: IdbOpenDbRequest = window()
      .expect("window not available")
      .indexed_db()
      .unwrap()
      .expect("idb not available")
      .open("my_db")
      .expect("Failed to get idb request");
    req.set_onerror(Some(&future.as_function()));
    let inner_ref = {
      let weak_ref = Rc::downgrade(&future.inner);
      req.set_onsuccess(Some(future.as_closure().as_ref().as_ref().unchecked_ref()));
      assert_eq!(weak_ref.upgrade().is_some(), true); // Assert inner_ref `Some`
      weak_ref
    };
    assert_eq!(inner_ref.upgrade().is_some(), true); // Assert inner_ref `Some`
    future.await.unwrap();
    assert_eq!(inner_ref.upgrade().is_none(), true); // Assert inner_ref `None`
  }
}
