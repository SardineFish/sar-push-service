use crate::smtp::{SMTPInner, Stream};
use std::io::{self, Read, Write};

pub trait Extension<'s, S: Stream> {
    fn name() -> &'static str;
    fn register(smtp: &'s mut SMTPInner<S>, params: &[String]) -> Self;
}