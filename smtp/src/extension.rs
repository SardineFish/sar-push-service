pub trait Extension {
    fn name() -> &'static str;
    fn register(params: &[&str]) -> Self;
}