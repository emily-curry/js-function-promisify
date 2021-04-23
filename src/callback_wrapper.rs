use crate::callback_kind::CallbackKind;
use crate::closure_kind::ClosureKind;
use js_sys::Function;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::task::Poll;
use std::task::Waker;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;

fn finish(state: &RefCell<CallbackWrapperInner>, val: Result<JsValue, JsValue>) {
  let task = {
    let mut state = state.borrow_mut();
    debug_assert!(state.cb.is_some());
    debug_assert!(state.result.is_none());
    if let Some(cb) = &state.cb {
      drop(cb)
    }
    state.result = Some(val);
    state.task.take()
  };

  if let Some(task) = task {
    task.wake()
  }
}

pub struct CallbackWrapper {
  inner: Rc<RefCell<CallbackWrapperInner>>,
}

impl CallbackWrapper {
  /// Creates a new `CallbackWrapper` wrapping a simple callback in the form `(data) => any`.
  /// The result of awaiting this wrapper will be Ok(data).
  pub fn new() -> Self {
    let func = |x| Ok(x);
    CallbackWrapper::from(CallbackKind::Arg1(Box::new(func)))
  }

  /// Creates a new `CallbackWrapper` wrapping a node-style callback in the form `(err, data) => any`.
  /// The result of awaiting this wrapper will be Ok(data) if `err` was null or undefined, or Err(err) otherwise.
  pub fn new_node() -> Self {
    let func = |err: JsValue, data: JsValue| {
      if err.is_null() || err.is_undefined() {
        return Ok(data);
      }
      Err(err)
    };
    CallbackWrapper::from(CallbackKind::Arg2(Box::new(func)))
  }

  pub fn from(cb: CallbackKind) -> Self {
    let inner = CallbackWrapperInner::new();

    let state = inner.clone();
    let ckind = match cb {
      CallbackKind::Arg0(mut f) => ClosureKind::Arg0(Closure::once(move || finish(&state, f()))),
      CallbackKind::Arg1(mut f) => {
        ClosureKind::Arg1(Closure::once(move |a1| finish(&state, f(a1))))
      }
      CallbackKind::Arg2(mut f) => {
        ClosureKind::Arg2(Closure::once(move |a1, a2| finish(&state, f(a1, a2))))
      }
      CallbackKind::Arg3(mut f) => ClosureKind::Arg3(Closure::once(move |a1, a2, a3| {
        finish(&state, f(a1, a2, a3))
      })),
      CallbackKind::Arg4(mut f) => ClosureKind::Arg4(Closure::once(move |a1, a2, a3, a4| {
        finish(&state, f(a1, a2, a3, a4))
      })),
      CallbackKind::Arg5(mut f) => ClosureKind::Arg5(Closure::once(move |a1, a2, a3, a4, a5| {
        finish(&state, f(a1, a2, a3, a4, a5))
      })),
    };

    inner.borrow_mut().cb = Some(Rc::new(ckind));
    let wrapper = Self { inner };

    wrapper
  }

  pub fn get_closure(&self) -> Rc<ClosureKind> {
    let inner = self.inner.borrow();
    let cb = inner
      .cb
      .as_ref()
      .expect("closure should've been defined on construction");
    let ptr = Rc::clone(&cb);
    ptr
  }

  pub fn get_function(&self) -> Function {
    let cb_ptr = self.get_closure();
    let js_func: JsValue = match cb_ptr.as_ref() {
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

struct CallbackWrapperInner {
  cb: Option<Rc<ClosureKind>>,
  result: Option<Result<JsValue, JsValue>>,
  task: Option<Waker>,
}

impl CallbackWrapperInner {
  fn new() -> Rc<RefCell<CallbackWrapperInner>> {
    Rc::new(RefCell::new(CallbackWrapperInner {
      cb: None,
      task: None,
      result: None,
    }))
  }
}

impl Future for CallbackWrapper {
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
