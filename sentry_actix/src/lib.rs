//! This is all taken from https://github.com/getsentry/symbolicator.
//! Currently axix-web 1.X is unsupported with the official sentry_actix middleware.
//! See https://github.com/getsentry/sentry-rust/issues/143

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::future::LocalBoxFuture;
use pin_project::pin_project;
use sentry::{Hub, Scope};

/// A future that binds a `Hub` to its execution.
#[pin_project]
#[derive(Debug)]
pub struct SentryFuture<F> {
    hub: Arc<Hub>,
    #[pin]
    future: F,
}

impl<F> SentryFuture<F> {
    /// Creates a new bound future with a `Hub`.
    #[allow(unused)]
    pub fn new(hub: Arc<Hub>, future: F) -> Self {
        Self { hub, future }
    }
}

impl<F> Future for SentryFuture<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        Hub::run(self.hub.clone(), || self.as_mut().project().future.poll(cx))
    }
}

/// Future extensions for Sentry.
pub trait SentryFutureExt: Sized {
    /// Binds a hub to the execution of this future.
    ///
    /// This ensures that the future is polled within the given hub.
    fn bind_hub<H>(self, hub: H) -> SentryFuture<Self>
    where
        H: Into<Arc<Hub>>,
    {
        SentryFuture {
            future: self,
            hub: hub.into(),
        }
    }
}

impl<F> SentryFutureExt for F where F: Future {}

/// Configures the sentry `Scope` with data from this object.
pub trait ToSentryScope {
    /// Writes data to the given scope.
    ///
    /// This can be called inside `sentry::configure_scope`. There is also a shorthand
    /// `.configure_scope` in this trait.
    fn to_scope(&self, scope: &mut Scope);

    /// Configures the current scope.
    fn configure_scope(&self) {
        sentry::configure_scope(|scope| self.to_scope(scope));
    }

    /// Configures the top scope on the given hub.
    fn configure_hub(&self, hub: &Hub) {
        hub.configure_scope(|scope| self.to_scope(scope));
    }
}

/// A dynamically dispatched future.
///
/// This future cannot be shared across threads, which makes it not eligible for the use in thread
/// pools.
pub type ResultFuture<T, E> = LocalBoxFuture<'static, Result<T, E>>;

mod middleware;
pub use middleware::Sentry as SentryMiddleware;
