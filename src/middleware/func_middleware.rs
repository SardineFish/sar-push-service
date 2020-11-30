use actix_web::{
    dev::{Service, Transform, ServiceRequest, ServiceResponse, MessageBody},
};
use futures_util::future::{ok, Ready, Future};
use std::task::{
    self,
    Poll
};
use std::pin::Pin;
use std::rc::Rc;
use std::cell::{RefCell, RefMut};

#[derive(Clone)]
pub struct FuncMiddleware<S, F> {
    func: fn(req: ServiceRequest, service: Rc<RefCell<S>>) -> F,
}

impl<S, B, F> FuncMiddleware<S, F>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: MessageBody,
    F: Future<Output = Result<ServiceResponse<B>, actix_web::Error>>
{
    pub fn from_func(func: fn(req: ServiceRequest, service: Rc<RefCell<S>>) -> F) -> Self{
        Self {
            func: func
        }
    }
    pub fn from_closure(func: impl FnMut(ServiceRequest, Rc<RefCell<S>>) -> F) -> Self {
        Self {
            func: func,
        }
    }
}

impl<S, B, F> Transform<S> for FuncMiddleware<S, F>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: MessageBody,
    F: Future<Output = Result<ServiceResponse<B>, actix_web::Error>> + 'static
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
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct FuncMiddlewareFuture<S, F> {
    func: fn(req: ServiceRequest, service: Rc<RefCell<S>>) -> F,
    service: Rc<RefCell<S>>,
}

impl<S, B, F> Service for FuncMiddlewareFuture<S, F>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: MessageBody, 
    F: Future<Output = Result<ServiceResponse<B>, actix_web::Error>> + 'static
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    fn poll_ready(&mut self, ctx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }
    fn call(&mut self, req: Self::Request) -> Self::Future {
        let service = self.service.clone();
        let func = self.func;
        Box::pin(async move {
            let result = func(req, service).await?;
            Ok(result)
        })
    }
}