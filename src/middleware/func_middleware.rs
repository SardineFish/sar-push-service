use actix_web::{
    dev::{Service, Transform, ServiceRequest, ServiceResponse, MessageBody},
};
use futures_util::future::{ok, Ready, Future};
use std::task::{
    self,
    Poll
};
use std::pin::Pin;

#[derive(Clone)]
pub struct FuncMiddleware<S, F> {
    func: fn(service: &mut S, req: ServiceRequest) -> F,
}

impl<S, B, F> FuncMiddleware<S, F>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody,
    F: Future<Output = Result<ServiceResponse<B>, actix_web::Error>>
{
    pub fn from_func(func:fn(service: &mut S, req: ServiceRequest) -> F) -> Self{
        Self {
            func: func
        }
    }
}

impl<S, B, F> Transform<S> for FuncMiddleware<S, F>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody,
    F: Future<Output = Result<ServiceResponse<B>, actix_web::Error>>
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = FuncMiddlewareFuture<S, F>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ok(FuncMiddlewareFuture {
            func: self.func,
            service: service,
        })
    }
}

pub struct FuncMiddlewareFuture<S, F> {
    func: fn(service: &mut S, req: ServiceRequest) -> F,
    service: S,
}

impl<S, B, F> Service for FuncMiddlewareFuture<S, F>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody, 
    F: Future<Output = Result<ServiceResponse<B>, actix_web::Error>>
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<F>>;
    fn poll_ready(&mut self, ctx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }
    fn call(&mut self, req: Self::Request) -> Self::Future {
        let func = self.func;
        let future = func(&mut self.service, req);
        Box::pin(future)
    }
}