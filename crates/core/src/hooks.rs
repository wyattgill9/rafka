/// Marker trait for hook implementations.
///
/// All hook traits inherit from this trait.
/// Implementations must be thread-safe (`Send + Sync`).
// pub trait Hook: Send + Sync {}

/// Marker trait for request types used in hooks.
///
/// Types implementing this trait must be thread-safe (`Send + Sync`).
// pub trait RequestBase: Send + Sync {}

/// Marker trait for response types used in hooks.
///
/// Types implementing this trait must be thread-safe (`Send + Sync`).
// pub trait ResponseBase: Send + Sync {}

/// Marker trait for hook result types.
///
/// This trait is used to enforce constraints on the `HookResult` associated types.
// pub trait HookResultBase {}

/// Trait for hooks that are invoked when a request is received.
///
/// Implementors can define their own `Request` and `HookResult` types.
pub trait OnRequestReceived: Send + Sync {
    /// The type representing a request.
    type Request: Send + Sync;

    /// The type representing the result of the hook execution.
    type HookResult: Send + Sync;

    /// Method invoked when a request is received.
    ///
    /// Implementations can inspect or modify the request, or influence processing flow.
    ///
    /// # Arguments
    ///
    /// * `request` - A reference to the request.
    ///
    /// # Returns
    ///
    /// A `HookResult` indicating how to proceed.
    fn on_request_received(&self, request: &Self::Request) -> Self::HookResult;
}

pub trait OnRequestExecuted: Send + Sync {
    type Request: Send + Sync;
    type Response: Send + Sync;
    type HookResult;

    fn on_request_executed(
        &self,
        request: &Self::Request,
        response: &Self::Response,
    ) -> Self::HookResult;
}

pub trait BeforeResponseSend: Send + Sync {
    type Response: Send + Sync;
    type HookResult;

    fn before_response_send(&self, response: &mut Self::Response) -> Self::HookResult;
}

pub trait AfterResponseSend: Send + Sync {
    type Response: Send + Sync;
    type HookResult;

    fn after_response_send(&self, response: &Self::Response) -> Self::HookResult;
}

/// A base hook result type that can be used by hook implementations.
///
/// Generic over the `Request`, `Response`, and error types.
///
/// # Type Parameters
///
/// * `Req` - The request type implementing `RequestBase`.
/// * `Resp` - The response type implementing `ResponseBase`.
/// * `E` - The error type
pub enum BaseHookResult<Req: Send + Sync, Resp: Send + Sync, E> {
    /// Continue with normal processing.
    Continue,
    /// Stop further processing.
    StopProcessing,
    /// Replace the current request with a modified one.
    ModifyRequest(Req),
    /// Replace the current response with a modified one.
    ModifyResponse(Resp),
    /// An error occurred during hook execution.
    Error(E),
}
