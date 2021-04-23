mod callback_kind;
mod callback_wrapper;
mod closure_kind;
mod promise_wrapper;

pub use callback_kind::CallbackKind;
pub use callback_wrapper::CallbackWrapper;
pub use closure_kind::ClosureKind;
pub use promise_wrapper::PromiseWrapper;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
