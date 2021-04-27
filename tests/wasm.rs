use js_function_promisify::CallbackFuture;
use std::rc::Rc;
use wasm_bindgen_test::*;
use web_sys::{window, IdbOpenDbRequest};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn readme_example() {
  let future = CallbackFuture::new();

  window()
    .unwrap()
    .set_timeout_with_callback_and_timeout_and_arguments_0(
      future
        .get_arg0(|| Ok("Hello future!".into()))
        .as_function()
        .as_ref(),
      500,
    )
    .unwrap();

  let result = future.await;
  assert_eq!(result.is_ok(), true); // Assert `Ok`
  assert_eq!(result.unwrap().as_string().unwrap(), "Hello future!"); // Assert the result exactly equals the string
}

#[wasm_bindgen_test]
async fn closure_dropped_after_await() {
  let future = CallbackFuture::new();
  let req: IdbOpenDbRequest = window()
    .expect("window not available")
    .indexed_db()
    .unwrap()
    .expect("idb not available")
    .open("my_db")
    .expect("Failed to get idb request");
  let weak_ref = {
    let cb = future.get_arg1(|a| Ok(a));
    let weak_ref = Rc::downgrade(&cb);
    req.set_onsuccess(Some(cb.as_function().as_ref()));
    assert_eq!(weak_ref.upgrade().is_some(), true); // Assert `Some`
    weak_ref
  };
  assert_eq!(weak_ref.upgrade().is_some(), true); // Assert `Some`
  future.await.unwrap();
  assert_eq!(weak_ref.upgrade().is_none(), true); // Assert `None`
}
