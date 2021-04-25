mod callback;
mod callback_wrapper;

pub use callback::{Callback, CallbackMarker};
pub use callback_wrapper::CallbackWrapper;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
