use my_core::hooks::{
    AfterResponseSend, BaseHookResult, BeforeResponseSend, OnRequestExecuted, OnRequestReceived,
};
use std::sync::Arc;

use crate::request::Request;
use crate::response::Response;

pub struct HookRegistry {
    pub on_request_received_hooks: Vec<
        Arc<
            dyn OnRequestReceived<
                Request = Request,
                HookResult = BaseHookResult<Request, Response, BrokerHookError>,
            >,
        >,
    >,
    pub on_request_executed_hooks: Vec<
        Arc<
            dyn OnRequestExecuted<
                Request = Request,
                Response = Response,
                HookResult = BaseHookResult<Request, Response, BrokerHookError>,
            >,
        >,
    >,
    pub before_response_send_hooks:
        Vec<Arc<dyn BeforeResponseSend<Response = Response, HookResult = HookResultType>>>,
    pub after_response_send_hooks:
        Vec<Arc<dyn AfterResponseSend<Response = Response, HookResult = HookResultType>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            on_request_received_hooks: Vec::new(),
            on_request_executed_hooks: Vec::new(),
            before_response_send_hooks: Vec::new(),
            after_response_send_hooks: Vec::new(),
        }
    }

    // Methods to add hooks
    pub fn add_on_request_received_hook(
        &mut self,
        hook: Arc<dyn OnRequestReceived<Request = Request, HookResult = HookResultType>>,
    ) {
        self.on_request_received_hooks.push(hook);
    }

    // Similar methods for other hooks...
}

// Define the HookResultType for this broker
pub type HookResultType = BaseHookResult<Request, Response, BrokerHookError>;

// Define BrokerHookError
#[derive(Debug)]
pub struct BrokerHookError {
    pub message: String,
}
