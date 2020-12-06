#![feature(str_split_once)]
#![feature(trait_alias)]

extern crate chrono;

mod command;
mod utils;
mod smtp;
pub mod error;
mod reply;
mod extension;
pub mod auth;
pub mod mail;
pub mod mime;
mod buffer;

pub use crate::smtp::SMTPClient;
pub use crate::auth::AuthCommand;
pub use crate::mime::MIMEBody;
pub use crate::mail::MailBuilder;
pub use crate::error::{Error, Result};