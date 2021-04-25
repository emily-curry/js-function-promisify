use js_function_promisify::CallbackKind;
use js_function_promisify::CallbackWrapper;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{IdbFactory, IdbOpenDbRequest, WorkerGlobalScope};
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn closure_dropped_after_await() {
  let wrapper = CallbackWrapper::new();
  let global: WorkerGlobalScope = js_sys::global().unchecked_into();
  let idb_fac: IdbFactory = global.indexed_db().unwrap().expect("idb not available");
  let req: IdbOpenDbRequest = idb_fac.open("my_db").expect("Failed to get idb request");
  let weak_ref = {
    let cl = wrapper.get_args1(|a| Ok(a));
    let weak_ref = Rc::downgrade(&cl);
    req.set_onsuccess(Some(cl.to_function().as_ref()));
    assert_eq!(weak_ref.upgrade().is_some(), true); // Assert `Some`
    weak_ref
  };
  assert_eq!(weak_ref.upgrade().is_some(), true); // Assert `Some`
  wrapper.await.unwrap();
  assert_eq!(weak_ref.upgrade().is_none(), true); // Assert `None`
}
