use core::cell::RefCell;
use js_sys::Function;
use std::fmt::Debug;
use std::future::Future;
use std::rc::Rc;
use std::task::Poll;
use std::task::Waker;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;

/// A `CallbackPair<F>` is a wrapper around a `wasm_bindgen::prelude::Closure<F>` which supports TODO:
#[derive(Debug)]
pub struct CallbackPair<A, B>
where
  A: 'static + ?Sized,
  B: 'static + ?Sized,
{
  inner: Rc<RefCell<CallbackPairInner<A, B>>>,
}

impl<A, B> CallbackPair<A, B>
where
  A: 'static + ?Sized,
  B: 'static + ?Sized,
{
  pub fn as_functions(&self) -> (Function, Function) {
    let left: JsValue = self
      .inner
      .borrow()
      .cb
      .as_ref()
      .unwrap()
      .as_ref()
      .0
      .as_ref()
      .into();
    let right: JsValue = self
      .inner
      .borrow()
      .cb
      .as_ref()
      .unwrap()
      .as_ref()
      .1
      .as_ref()
      .into();
    (left.into(), right.into())
  }

  pub fn as_closures(&self) -> Rc<(Closure<A>, Closure<B>)> {
    Rc::clone(self.inner.borrow().cb.as_ref().unwrap())
  }
}

impl<A, B> Future for CallbackPair<A, B>
where
  A: 'static + ?Sized,
  B: 'static + ?Sized,
{
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

impl<A, B> From<(A, B)> for CallbackPair<dyn FnMut(), dyn FnMut()>
where
  A: 'static + FnMut() -> Result<JsValue, JsValue>,
  B: 'static + FnMut() -> Result<JsValue, JsValue>,
{
  fn from(cb: (A, B)) -> Self {
    let inner = CallbackPairInner::new();
    let state = Rc::clone(&inner);
    let mut a = cb.0;
    let left = Closure::once(move || CallbackPairInner::finish(&state, a()));
    let state = Rc::clone(&inner);
    let mut b = cb.1;
    let right = Closure::once(move || CallbackPairInner::finish(&state, b()));
    let ptr = Rc::new((left, right));
    inner.borrow_mut().cb = Some(ptr);
    CallbackPair { inner }
  }
}

impl CallbackPair<dyn FnMut(), dyn FnMut()> {
  pub fn from_arg0<A, B>(a: A, b: B) -> CallbackPair<dyn FnMut(), dyn FnMut()>
  where
    A: 'static + FnMut() -> Result<JsValue, JsValue>,
    B: 'static + FnMut() -> Result<JsValue, JsValue>,
  {
    CallbackPair::from((a, b))
  }
}

#[derive(Debug)]
pub struct CallbackPairInner<A, B>
where
  A: 'static + ?Sized,
  B: 'static + ?Sized,
{
  cb: Option<Rc<(Closure<A>, Closure<B>)>>,
  result: Option<Result<JsValue, JsValue>>,
  task: Option<Waker>,
}

impl<A, B> CallbackPairInner<A, B>
where
  A: 'static + ?Sized,
  B: 'static + ?Sized,
{
  pub fn new() -> Rc<RefCell<CallbackPairInner<A, B>>> {
    Rc::new(RefCell::new(CallbackPairInner {
      cb: None,
      task: None,
      result: None,
    }))
  }

  pub fn finish(state: &RefCell<CallbackPairInner<A, B>>, val: Result<JsValue, JsValue>) {
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
  use crate::CallbackPair;
  use std::rc::Rc;
  use wasm_bindgen_test::*;
  use web_sys::{window, IdbOpenDbRequest};

  wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

  #[wasm_bindgen_test]
  async fn inner_dropped_after_await() {
    let future = CallbackPair::from_arg0(|| Ok("".into()), || Err("".into()));
    let req: IdbOpenDbRequest = window()
      .expect("window not available")
      .indexed_db()
      .unwrap()
      .expect("idb not available")
      .open("my_db")
      .expect("Failed to get idb request");
    let functions = future.as_functions();
    req.set_onerror(Some(&functions.1));
    let inner_ref = {
      let weak_ref = Rc::downgrade(&future.inner);
      req.set_onsuccess(Some(&functions.0));
      assert_eq!(weak_ref.upgrade().is_some(), true); // Assert inner_ref `Some`
      weak_ref
    };
    assert_eq!(inner_ref.upgrade().is_some(), true); // Assert inner_ref `Some`
    future.await.unwrap();
    assert_eq!(inner_ref.upgrade().is_none(), true); // Assert inner_ref `None`
  }
}
