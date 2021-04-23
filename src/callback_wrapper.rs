use js_sys::Function;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::task::Poll;
use std::task::Waker;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};

pub struct CallbackWrapper {
  inner: Rc<RefCell<CallbackWrapperInner>>,
}

impl CallbackWrapper {
  pub fn new() -> Self {
    let func = &|x| Ok(x);
    CallbackWrapper::from_arg1(func)
  }

  pub fn from_arg1<F>(f: &'static F) -> Self
  where
    F: Fn(JsValue) -> Result<JsValue, JsValue>,
  {
    let inner = Rc::new(RefCell::new(CallbackWrapperInner {
      cb: None,
      task: None,
      result: None,
    }));

    let cb: Closure<dyn FnMut(JsValue)> = {
      let state = inner.clone();
      Closure::once(move |val| CallbackWrapper::finish(&state, f(val)))
    };
    inner.borrow_mut().cb = Some(Rc::new(cb));
    let wrapper = Self { inner };

    wrapper
  }

  pub fn get_closure(&self) -> Rc<Closure<dyn FnMut(JsValue)>> {
    let inner = self.inner.borrow();
    let cb = inner.cb.as_ref().unwrap();
    let ptr = Rc::clone(&cb);
    ptr
  }

  pub fn get_function(&self) -> Function {
    let cb_ptr = self.get_closure();
    let js_func: JsValue = cb_ptr.as_ref().as_ref().into();
    let func: Function = js_func.into();
    func
  }

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
}

struct CallbackWrapperInner {
  cb: Option<Rc<Closure<dyn FnMut(JsValue)>>>,
  result: Option<Result<JsValue, JsValue>>,
  task: Option<Waker>,
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
