
#[macro_use]
mod func_middleware;
mod auth;

pub use auth::authentication;
pub use func_middleware::{FuncMiddleware, ServiceT, AsyncFactoryRtn, AsyncMiddlewareRtn};