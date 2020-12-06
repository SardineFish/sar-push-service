use crate::smtp::{SMTPInner, Stream};

pub trait Extension<'s, S: Stream> {
    fn name() -> &'static str;
    fn register(smtp: &'s mut SMTPInner<S>, params: &[String]) -> Self;
}