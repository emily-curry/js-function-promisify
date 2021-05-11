# js-function-promisify

![docs](https://docs.rs/js-function-promisify/badge.svg)

This package provides utilities for working with `js_sys::Function` callbacks in async rust the same way you might in javascript. For example, to wait on a timeout to complete in javascript, one might write:

```js
const promise = new Promise((resolve) => {
  window.setTimeout(() => { resolve('Hello future!'); }, 500);
});
const result = await promise;
result === 'Hello future!'; // true
```

To accomplish the same thing with `js-function-promisify`:

```rust
let future = Callback::new(|| Ok("Hello future!".into()));
web_sys::window()
  .unwrap()
  .set_timeout_with_callback_and_timeout_and_arguments_0(
    future.as_function().as_ref(),
    500)
  .unwrap();
let result = future.await; // result: Result<JsValue, JsValue>
assert_eq!(result.unwrap().as_string().unwrap(), "Hello future!"); // 🦀
```

## Usage

TODO: Document common ways to use Callback.
TODO: Document gotchas, primarily use of Closure::once and deallocation on first call.

## Contributing

Contributions are welcome! Feel free to open an issue if you find a bug or have a feature request. Additionally, feedback on the public API is more than welcome, as this is the first library I've published and I'm still getting a feel for what's idiomatic in rust.

### Testing

Using `wasm-pack` is the easiest way to run the tests in this repository. Those folks have some great documentation on it [here](https://rustwasm.github.io/docs/wasm-pack/commands/test.html) and [here](https://rustwasm.github.io/docs/wasm-bindgen/wasm-bindgen-test/usage.html), but as an example one may run the following to run all tests in the library:

```sh
wasm-pack test --headless --firefox
```