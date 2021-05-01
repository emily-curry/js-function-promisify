mod callback;
mod callback_pair;

pub use callback::Callback;
pub use callback_pair::CallbackPair;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
