// src/middleware/rate_limiter.rs

use actix_web::{
    body::EitherBody, // Import EitherBody
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::{ok, Ready};
use std::{
    collections::HashMap,
    marker::PhantomData, // Import PhantomData
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

/// RateLimiter struct to hold configuration and client data
#[derive(Clone)]
pub struct RateLimiter {
    max_requests: u64,
    window: Duration,
    clients: Arc<Mutex<HashMap<String, ClientData>>>,
}

struct ClientData {
    requests: u64,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: u64, window: Duration) -> Self {
        RateLimiter {
            max_requests,
            window,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Checks if the request from the given IP is allowed
    async fn is_allowed(&self, ip: &str) -> bool {
        let mut clients = self.clients.lock().await;
        let now = Instant::now();

        let client = clients.entry(ip.to_string()).or_insert(ClientData {
            requests: 0,
            window_start: now,
        });

        if now.duration_since(client.window_start) > self.window {
            client.requests = 1;
            client.window_start = now;
            true
        } else if client.requests < self.max_requests {
            client.requests += 1;
            true
        } else {
            false
        }
    }
}

/// Middleware to enforce rate limiting
pub struct RateLimiterMiddleware {
    rate_limiter: RateLimiter,
}

impl RateLimiterMiddleware {
    pub fn new(rate_limiter: RateLimiter) -> Self {
        RateLimiterMiddleware { rate_limiter }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiterMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>; // Updated here
    type Error = Error;
    type Transform = RateLimiterMiddlewareService<S, B>;
    type InitError = (); // Added this line
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimiterMiddlewareService {
            service: Arc::new(service),
            rate_limiter: self.rate_limiter.clone(),
            _marker: PhantomData, // Initialize PhantomData
        })
    }
}

/// Service struct for RateLimiterMiddleware
pub struct RateLimiterMiddlewareService<S, B> {
    service: Arc<S>,
    rate_limiter: RateLimiter,
    _marker: PhantomData<B>, // Added PhantomData to utilize type parameter B
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddlewareService<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>; // Updated here
    type Error = Error;
    type Future = futures_util::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Arc::clone(&self.service);
        let rate_limiter = self.rate_limiter.clone();
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        Box::pin(async move {
            if rate_limiter.is_allowed(&ip).await {
                let res = service.call(req).await?;
                Ok(res.map_into_left_body()) // Returns EitherBody<B>
            } else {
                let response = HttpResponse::TooManyRequests()
                    .body("Too many requests")
                    .map_into_right_body(); // Returns EitherBody<B>
                Ok(req.into_response(response))
            }
        })
    }
}
