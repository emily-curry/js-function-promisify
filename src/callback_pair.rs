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
  pub fn new<X, Y>(x: X, y: Y) -> CallbackPair<A, B>
  where
    Self: From<(X, Y)>,
  {
    Self::from((x, y))
  }

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

/// The Default impl for CallbackPair creates a pair of single-arg `(resolve, reject)` callbacks,
/// similar to the javascript Promise contsructor.
impl Default for CallbackPair<dyn FnMut(JsValue), dyn FnMut(JsValue)> {
  fn default() -> Self {
    Self::from((|data| Ok(data), |err| Err(err)))
  }
}

/// Standard impl of Future for CallbackPair.
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

/// A utility macro for generating every possible implementation of `From<A, B> for CallbackPair`.
macro_rules! from_impl {
  // The main arm of this macro. Generates a single From impl for CallbackPair.
  // a - The list of parameter types that FnMut A takes.
  // b - The list of parameter types that FnMut B takes.
  // alist - The argument list of A.
  // blist - The argument list of B.
  (($($a:ty),*), ($($b:ty),*), ($($alist:ident),*), ($($blist:ident),*)) => {
    impl<A, B> From<(A, B)> for CallbackPair<dyn FnMut($($a,)*), dyn FnMut($($b,)*)>
    where
      A: 'static + FnOnce($($a,)*) -> Result<JsValue, JsValue>,
      B: 'static + FnOnce($($b,)*) -> Result<JsValue, JsValue>,
    {
      fn from(cb: (A, B)) -> Self {
        let inner = CallbackPairInner::new();
        let state = Rc::clone(&inner);
        let cb0 = cb.0;
        let left = Closure::once(move |$($alist),*| CallbackPairInner::finish(&state, cb0($($alist),*)));
        let state = Rc::clone(&inner);
        let cb1 = cb.1;
        let right = Closure::once(move |$($blist),*| CallbackPairInner::finish(&state, cb1($($blist),*)));
        let ptr = Rc::new((left, right));
        inner.borrow_mut().cb = Some(ptr);
        CallbackPair { inner }
      }
    }
  };
  // Shorthand for the main arm. Based on the argument list, generate the parameter types (always JsValue) for that list.
  (($($a:ident,)*), ($($b:ident,)*)) => {
    from_impl!(($(from_impl!(@rep $a JsValue)),*), ($(from_impl!(@rep $b JsValue)),*), ($($a),*), ($($b),*));
  };
  // Recursively generates a set of impls where the left arg list stays the same and the right arg list gets smaller.
  (@left ($($a:ident,)*); $head:ident $($tail:tt)*) => {
    from_impl!(($($a,)*), ($head, $($tail,)*));
    from_impl!(@left ($($a,)*); $($tail)*);
  };
  // Recursively generates a set of impls where the right arg list stays the same and the left arg list gets smaller.
  (@right ($($b:ident,)*); $head:ident $($tail:tt)*) => {
    from_impl!(($head, $($tail,)*), ($($b,)*));
    from_impl!(@right ($($b,)*); $($tail)*);
  };
  // For a list of identifiers, creates every set of possible combinations of those identifiers and generates From<A, B> impls for them.
  ($head:ident $($tail:tt)*) => {
    // Generate a From impl for the full set of arguments on both sides.
    from_impl!(($head, $($tail,)*), ($head, $($tail,)*));
    // Using the same set of arguments on the left side, recursively generate a From impl for every possible set of args on the right.
    from_impl!(@left ($head, $($tail,)*); $($tail)*);
    // An empty arg list will never be generated, so impl it here.
    from_impl!(($head, $($tail,)*), ());
    // Using the same set of arguments on the right side, recursively generate a From impl for every possible set of args on the left.
    from_impl!(@right ($head, $($tail,)*); $($tail)*);
    // An empty arg list will never be generated, so impl it here.
    from_impl!((), ($head, $($tail,)*));
    // Recurse inwards, generating the same definitions with one less argument.
    from_impl!($($tail)*);
  };
  // Utility for replacing anything with a type.
  (@rep $_t:tt $sub:ty) => {
    $sub
  };
  // Empty arms for handling the end of recursion.
  () => {
    from_impl!((), ());
  };
  (@left ($($a:ident,)*); ) => {};
  (@right ($($b:ident,)*); ) => {};
}

from_impl!(a0 a1 a2 a3 a4 a5 a6); // Generate From impls for every possible permutation of arguments in either callback, up to 7.

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
  use wasm_bindgen::JsCast;
  use wasm_bindgen_test::*;
  use web_sys::{window, IdbOpenDbRequest};

  wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

  /// We could write a macro for this, but then I wouldn't be totally confident it captured every permutation.
  /// For now, we relish in the beauty of our christmas tree.
  #[wasm_bindgen_test]
  #[rustfmt::skip]
  fn should_compile_with_any_args() {
    let _r = CallbackPair::new(|| Ok("".into()), || Err("".into()));
    let _r = CallbackPair::new(|_a| Ok("".into()), || Err("".into()));
    let _r = CallbackPair::new(|_a, _b| Ok("".into()), || Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c| Ok("".into()), || Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d| Ok("".into()), || Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e| Ok("".into()), || Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f| Ok("".into()), || Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f, _g| Ok("".into()), || Err("".into()));
    let _r = CallbackPair::new(|| Ok("".into()), |_a| Err("".into()));
    let _r = CallbackPair::new(|_a| Ok("".into()), |_a| Err("".into()));
    let _r = CallbackPair::new(|_a, _b| Ok("".into()), |_a| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c| Ok("".into()), |_a| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d| Ok("".into()), |_a| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e| Ok("".into()), |_a| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f| Ok("".into()), |_a| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f, _g| Ok("".into()), |_a| Err("".into()));
    let _r = CallbackPair::new(|| Ok("".into()), |_a, _b| Err("".into()));
    let _r = CallbackPair::new(|_a| Ok("".into()), |_a, _b| Err("".into()));
    let _r = CallbackPair::new(|_a, _b| Ok("".into()), |_a, _b| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c| Ok("".into()), |_a, _b| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d| Ok("".into()), |_a, _b| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e| Ok("".into()), |_a, _b| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f| Ok("".into()), |_a, _b| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f, _g| Ok("".into()), |_a, _b| Err("".into()));
    let _r = CallbackPair::new(|| Ok("".into()), |_a, _b, _c| Err("".into()));
    let _r = CallbackPair::new(|_a| Ok("".into()), |_a, _b, _c| Err("".into()));
    let _r = CallbackPair::new(|_a, _b| Ok("".into()), |_a, _b, _c| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c| Ok("".into()), |_a, _b, _c| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d| Ok("".into()), |_a, _b, _c| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e| Ok("".into()), |_a, _b, _c| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f| Ok("".into()), |_a, _b, _c| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f, _g| Ok("".into()), |_a, _b, _c| Err("".into()));
    let _r = CallbackPair::new(|| Ok("".into()), |_a, _b, _c, _d| Err("".into()));
    let _r = CallbackPair::new(|_a| Ok("".into()), |_a, _b, _c, _d| Err("".into()));
    let _r = CallbackPair::new(|_a, _b| Ok("".into()), |_a, _b, _c, _d| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c| Ok("".into()), |_a, _b, _c, _d| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d| Ok("".into()), |_a, _b, _c, _d| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e| Ok("".into()), |_a, _b, _c, _d| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f| Ok("".into()), |_a, _b, _c, _d| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f, _g| Ok("".into()), |_a, _b, _c, _d| Err("".into()));
    let _r = CallbackPair::new(|| Ok("".into()), |_a, _b, _c, _d, _e| Err("".into()));
    let _r = CallbackPair::new(|_a| Ok("".into()), |_a, _b, _c, _d, _e| Err("".into()));
    let _r = CallbackPair::new(|_a, _b| Ok("".into()), |_a, _b, _c, _d, _e| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c| Ok("".into()), |_a, _b, _c, _d, _e| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d| Ok("".into()), |_a, _b, _c, _d, _e| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e| Ok("".into()), |_a, _b, _c, _d, _e| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f| Ok("".into()), |_a, _b, _c, _d, _e| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f, _g| Ok("".into()), |_a, _b, _c, _d, _e| Err("".into()));
    let _r = CallbackPair::new(|| Ok("".into()), |_a, _b, _c, _d, _e, _f| Err("".into()));
    let _r = CallbackPair::new(|_a| Ok("".into()), |_a, _b, _c, _d, _e, _f| Err("".into()));
    let _r = CallbackPair::new(|_a, _b| Ok("".into()), |_a, _b, _c, _d, _e, _f| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c| Ok("".into()), |_a, _b, _c, _d, _e, _f| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d| Ok("".into()), |_a, _b, _c, _d, _e, _f| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e| Ok("".into()), |_a, _b, _c, _d, _e, _f| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f| Ok("".into()), |_a, _b, _c, _d, _e, _f| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f, _g| Ok("".into()), |_a, _b, _c, _d, _e, _f| Err("".into()));
    let _r = CallbackPair::new(|| Ok("".into()), |_a, _b, _c, _d, _e, _f, _g| Err("".into()));
    let _r = CallbackPair::new(|_a| Ok("".into()), |_a, _b, _c, _d, _e, _f, _g| Err("".into()));
    let _r = CallbackPair::new(|_a, _b| Ok("".into()), |_a, _b, _c, _d, _e, _f, _g| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c| Ok("".into()), |_a, _b, _c, _d, _e, _f, _g| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d| Ok("".into()), |_a, _b, _c, _d, _e, _f, _g| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e| Ok("".into()), |_a, _b, _c, _d, _e, _f, _g| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f| Ok("".into()), |_a, _b, _c, _d, _e, _f, _g| Err("".into()));
    let _r = CallbackPair::new(|_a, _b, _c, _d, _e, _f, _g| Ok("".into()), |_a, _b, _c, _d, _e, _f, _g| Err("".into()));
  }

  #[wasm_bindgen_test]
  async fn inner_dropped_after_await() {
    let future = CallbackPair::new(|| Ok("".into()), || Err("".into()));
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

  #[wasm_bindgen_test]
  async fn closure_dropped_after_await() {
    let future = CallbackPair::new(|| Ok("".into()), || Err("".into()));
    let req: IdbOpenDbRequest = window()
      .expect("window not available")
      .indexed_db()
      .unwrap()
      .expect("idb not available")
      .open("my_db")
      .expect("Failed to get idb request");
    let wref = {
      let closures = future.as_closures();
      let weak_ref = Rc::downgrade(&closures);
      req.set_onsuccess(Some(closures.0.as_ref().as_ref().unchecked_ref()));
      req.set_onerror(Some(closures.1.as_ref().as_ref().unchecked_ref()));
      assert_eq!(weak_ref.upgrade().is_some(), true); // Assert resolve_ref `Some`
      weak_ref
    };
    assert_eq!(wref.upgrade().is_some(), true); // Assert resolve_ref `Some`
    future.await.unwrap();
    assert_eq!(wref.upgrade().is_none(), true); // Assert resolve_ref `None`
  }

  #[wasm_bindgen_test]
  async fn new_promise_left_resolve() {
    let future = CallbackPair::default();
    web_sys::window()
      .unwrap()
      .set_timeout_with_callback_and_timeout_and_arguments_0(future.as_functions().0.as_ref(), 200)
      .unwrap();
    let result = future.await;
    assert_eq!(result.is_ok(), true); // Assert is `Ok`
  }

  #[wasm_bindgen_test]
  async fn new_promise_right_reject() {
    let future = CallbackPair::default();
    web_sys::window()
      .unwrap()
      .set_timeout_with_callback_and_timeout_and_arguments_0(future.as_functions().1.as_ref(), 200)
      .unwrap();
    let result = future.await;
    assert_eq!(result.is_err(), true); // Assert is `Err`
  }
}
