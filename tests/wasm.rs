use js_function_promisify::Callback;
use js_function_promisify::CallbackPair;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{window, IdbOpenDbRequest};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn readme_example() {
  let future = Callback::from(|| Ok("Hello future!".into()));

  window()
    .unwrap()
    .set_timeout_with_callback_and_timeout_and_arguments_0(future.as_function().as_ref(), 500)
    .unwrap();

  let result = future.await;
  assert_eq!(result.is_ok(), true); // Assert `Ok`
  assert_eq!(result.unwrap().as_string().unwrap(), "Hello future!"); // Assert the result exactly equals the string
}

#[wasm_bindgen_test]
async fn closure_dropped_after_await() {
  let future = Callback::from_arg0(|| Ok("".into()));
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

#[wasm_bindgen_test]
async fn pair_closure_dropped_after_await() {
  let future = CallbackPair::from_arg0(|| Ok("".into()), || Err("".into()));
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
