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
  pub fn new<X>(closure: X) -> Callback<F>
  where
    Self: From<X>,
  {
    Self::from(closure)
  }

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

/// The Default impl for Callback creates a single-arg callback, whose Result is always Ok.
impl Default for Callback<dyn FnMut(JsValue)> {
  fn default() -> Self {
    Self::from(|data| Ok(data))
  }
}

impl Callback<dyn FnMut(JsValue, JsValue)> {
  /// Creates a node-style callback with the args `(err, data)`. If err is null or undefined,
  /// the Result is Ok(data). Otherwise, it is Err(err).
  pub fn default_node() -> Self {
    Self::from(|err: JsValue, data: JsValue| {
      if err.is_null() || err.is_undefined() {
        return Ok(data);
      }
      Err(err)
    })
  }
}

/// Standard Future impl for Callback<T>
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

/// A utility macro for generating every possible implementation of `From<A> for Callback`.
macro_rules! from_impl {
  // The main arm of this macro. Generates a single From impl for Callback.
  // a - The list of parameter types that FnMut A takes.
  // alist - The argument list of A.
  (($($a:ty),*), ($($alist:ident),*)) => {
    impl<A> From<A> for Callback<dyn FnMut($($a,)*)>
    where
      A: 'static + FnOnce($($a,)*) -> Result<JsValue, JsValue>,
    {
      fn from(cb: A) -> Self {
        let inner = CallbackInner::new();
        let state = Rc::clone(&inner);
        let closure = Closure::once(move |$($alist),*| CallbackInner::finish(&state, cb($($alist),*)));
        let ptr = Rc::new(closure);
        inner.borrow_mut().cb = Some(ptr);
        Callback { inner }
      }
    }
  };
  // Shorthand for the main arm. Based on the argument list, generate the parameter types (always JsValue) for that list.
  (($($a:ident,)*)) => {
    from_impl!(($(from_impl!(@rep $a JsValue)),*), ($($a),*));
  };
  // For a list of identifiers, recursively generates a From impl for that list and every list with less args.
  ($head:ident $($tail:tt)*) => {
    // Generate a From impl for the full set of arguments.
    from_impl!(($head, $($tail,)*));
    // Recurse inwards, generating the same definitions with one less argument.
    from_impl!($($tail)*);
  };
  // Utility for replacing anything with a type.
  (@rep $_t:tt $sub:ty) => {
    $sub
  };
  // Empty arms for handling the end of recursion.
  () => {
    from_impl!(());
  };
}

from_impl!(a0 a1 a2 a3 a4 a5 a6); // Generate From impls for each list of arguments, up to 7.

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
  use js_sys::Function;
  use std::rc::Rc;
  use wasm_bindgen::prelude::*;
  use wasm_bindgen::JsCast;
  use wasm_bindgen_test::*;
  use web_sys::{window, IdbOpenDbRequest};

  wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

  /// Not quite as beatiful as the CallbackPair test, but still important to enumerate every expected valid From impl.
  #[wasm_bindgen_test]
  #[rustfmt::skip]
  fn should_compile_with_any_args() {
    let _r = Callback::new(|| Ok("".into()));
    let _r = Callback::new(|_a| Ok("".into()));
    let _r = Callback::new(|_a, _b| Ok("".into()));
    let _r = Callback::new(|_a, _b, _c| Ok("".into()));
    let _r = Callback::new(|_a, _b, _c, _d| Ok("".into()));
    let _r = Callback::new(|_a, _b, _c, _d, _e| Ok("".into()));
    let _r = Callback::new(|_a, _b, _c, _d, _e, _f| Ok("".into()));
    let _r = Callback::new(|_a, _b, _c, _d, _e, _f, _g| Ok("".into()));
  }

  #[wasm_bindgen_test]
  async fn inner_dropped_after_await() {
    let future = Callback::default();
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

  #[wasm_bindgen_test]
  async fn closure_dropped_after_await() {
    let future = Callback::default();
    let req: IdbOpenDbRequest = window()
      .expect("window not available")
      .indexed_db()
      .unwrap()
      .expect("idb not available")
      .open("my_db")
      .expect("Failed to get idb request");
    req.set_onerror(Some(future.as_closure().as_ref().as_ref().unchecked_ref()));
    let resolve_ref = {
      let weak_ref = Rc::downgrade(&future.as_closure());
      req.set_onsuccess(Some(future.as_closure().as_ref().as_ref().unchecked_ref()));
      assert_eq!(weak_ref.upgrade().is_some(), true); // Assert resolve_ref `Some`
      weak_ref
    };
    assert_eq!(resolve_ref.upgrade().is_some(), true); // Assert resolve_ref `Some`
    future.await.unwrap();
    assert_eq!(resolve_ref.upgrade().is_none(), true); // Assert resolve_ref `None`
  }

  #[wasm_bindgen(
    inline_js = "export function extern_node_success_null(cb) { cb(null, 'success') }; 
    export function extern_node_success_undefined(cb) { cb(undefined, 'success') };
    export function extern_node_failure(cb) { cb('failure', 'success') };"
  )]
  extern "C" {
    fn extern_node_success_null(cb: &Function);
    fn extern_node_success_undefined(cb: &Function);
    fn extern_node_failure(cb: &Function);
  }

  #[wasm_bindgen_test]
  async fn node_ok_if_arg0_null() {
    let future = Callback::default_node();
    extern_node_success_null(future.as_function().as_ref());
    let result = future.await;
    assert_eq!(result.is_ok(), true); // Assert is `Ok`
    assert_eq!(result.unwrap(), "success");
  }

  #[wasm_bindgen_test]
  async fn node_ok_if_arg0_undefined() {
    let future = Callback::default_node();
    extern_node_success_undefined(future.as_function().as_ref());
    let result = future.await;
    assert_eq!(result.is_ok(), true); // Assert is `Ok`
    assert_eq!(result.unwrap(), "success");
  }

  #[wasm_bindgen_test]
  async fn node_err_if_arg0_defined() {
    let future = Callback::default_node();
    extern_node_failure(future.as_function().as_ref());
    let result = future.await;
    assert_eq!(result.is_err(), true); // Assert is `Err`
    assert_eq!(result.unwrap_err(), "failure");
  }
}
