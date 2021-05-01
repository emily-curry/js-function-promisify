use js_function_promisify::Callback;
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn readme_example() {
  let future = Callback::new(|| Ok("Hello future!".into()));
  web_sys::window()
    .unwrap()
    .set_timeout_with_callback_and_timeout_and_arguments_0(future.as_function().as_ref(), 500)
    .unwrap();
  let result = future.await; // result: Result<JsValue, JsValue>
  assert_eq!(result.is_ok(), true); // Assert `Ok`
  assert_eq!(result.unwrap().as_string().unwrap(), "Hello future!"); // Assert the result exactly equals the string
}
