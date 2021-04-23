use js_sys::Function;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::task::Poll;
use std::task::Waker;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;

fn finish(state: &RefCell<PromiseWrapperInner>, val: Result<JsValue, JsValue>) {
  let task = {
    let mut state = state.borrow_mut();
    debug_assert!(state.resolve.is_some());
    debug_assert!(state.reject.is_some());
    debug_assert!(state.result.is_none());
    if let Some(resolve) = &state.resolve {
      drop(resolve)
    }
    if let Some(reject) = &state.reject {
      drop(reject)
    }
    state.result = Some(val);
    state.task.take()
  };

  if let Some(task) = task {
    task.wake()
  }
}

pub struct PromiseWrapper {
  inner: Rc<RefCell<PromiseWrapperInner>>,
}

impl PromiseWrapper {
  pub fn new() -> Self {
    let resolve = |data| Ok(data);
    let reject = |err| Ok(err);
    PromiseWrapper::from(Box::new(resolve), Box::new(reject))
  }

  pub fn from(
    mut resolve: Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
    mut reject: Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
  ) -> Self {
    let inner = PromiseWrapperInner::new();

    let resolve_closure = {
      let state = inner.clone();
      Closure::once(move |data| finish(&state, resolve(data)))
    };
    inner.borrow_mut().resolve = Some(Rc::new(resolve_closure));

    let reject_closure = {
      let state = inner.clone();
      Closure::once(move |err| finish(&state, reject(err)))
    };
    inner.borrow_mut().reject = Some(Rc::new(reject_closure));

    let wrapper = Self { inner };

    wrapper
  }

  pub fn get_resolve_closure(&self) -> Rc<Closure<dyn FnMut(JsValue)>> {
    let inner = self.inner.borrow();
    let cb = inner
      .resolve
      .as_ref()
      .expect("closure should've been defined on construction");
    let ptr = Rc::clone(&cb);
    ptr
  }

  pub fn get_resolve_function(&self) -> Function {
    let cb_ptr = self.get_resolve_closure();
    let js_func: JsValue = cb_ptr.as_ref().as_ref().into();
    let func: Function = js_func.into();
    func
  }

  pub fn get_reject_closure(&self) -> Rc<Closure<dyn FnMut(JsValue)>> {
    let inner = self.inner.borrow();
    let cb = inner
      .reject
      .as_ref()
      .expect("closure should've been defined on construction");
    let ptr = Rc::clone(&cb);
    ptr
  }

  pub fn get_reject_function(&self) -> Function {
    let cb_ptr = self.get_reject_closure();
    let js_func: JsValue = cb_ptr.as_ref().as_ref().into();
    let func: Function = js_func.into();
    func
  }
}

struct PromiseWrapperInner {
  resolve: Option<Rc<Closure<dyn FnMut(JsValue)>>>,
  reject: Option<Rc<Closure<dyn FnMut(JsValue)>>>,
  result: Option<Result<JsValue, JsValue>>,
  task: Option<Waker>,
}

impl PromiseWrapperInner {
  fn new() -> Rc<RefCell<PromiseWrapperInner>> {
    Rc::new(RefCell::new(PromiseWrapperInner {
      resolve: None,
      reject: None,
      task: None,
      result: None,
    }))
  }
}

impl Future for PromiseWrapper {
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
