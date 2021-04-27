mod callback;
mod callback_future;

pub use callback::{Callback, CallbackMarker};
pub use callback_future::CallbackFuture;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
