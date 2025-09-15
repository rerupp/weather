//! The HTTP client that calls the Visual Crossing endpoint.
use reqwest::{
    // use the blocking API since the rest client is async.
    blocking::{Client, Request, RequestBuilder},
    StatusCode,
    Url,
};
use std::{
    cell::RefCell,
    sync::Arc,
    thread::{spawn, JoinHandle},
};

/// The result of a request made by the Rest client.
///
#[derive(Debug)]
pub enum RestClientResult {
    /// The body of the response.
    Body(Vec<u8>),
    /// The error if the underlying client panics.
    ClientPanic(String),
    /// The error if there is a problem executing the Rest call.
    ExecuteError(String),
    /// The error if there is a problem receiving the response body.
    ResponseError(String),
    /// The HTTP status code returned from the endpoint.
    HttpStatusCode(u16),
}

#[derive(Debug)]
/// The handle to the underlying thread join handle.
///
pub struct RestClientHandle {
    /// The underlying thread join handle and result.
    client_handle: RefCell<Option<JoinHandle<RestClientResult>>>,
}
impl RestClientHandle {
    /// Create a new instance of the client handle.
    ///
    fn new(client_handle: JoinHandle<RestClientResult>) -> Self {
        Self { client_handle: RefCell::new(Some(client_handle)) }
    }
    /// Check if the client has finished. If `true` is returned the next call to [get](Self::get()) will not block.
    ///
    pub fn is_finished(&self) -> bool {
        match self.client_handle.borrow().as_ref() {
            None => true,
            Some(join_handle) => join_handle.is_finished(),
        }
    }
    /// Get the result of the client request. This will block until the underlying thread exits.
    ///
    pub fn get(&self) -> RestClientResult {
        if self.client_handle.borrow().is_none() {
            log::error!("Rest client result already consumed.");
            RestClientResult::Body(vec![])
        } else {
            let client_result = self.client_handle.take().unwrap().join();
            if client_result.is_err() {
                RestClientResult::ClientPanic(format!("{:?}", client_result.err()))
            } else {
                client_result.unwrap()
            }
        }
    }
}

/// The asynchronous Rest client.
///
#[derive(Debug, Default)]
pub struct RestClient(
    /// The client is shared between threads so use a thread-safe reference.
    Arc<Client>,
);
impl RestClient {
    /// Create the asynchronous Rest client.
    ///
    pub fn new(client: Client) -> Self {
        Self(Arc::new(client))
    }
    /// Get the base URL of the Rest client endpoint.
    ///
    pub fn get(&self, url: Url) -> RequestBuilder {
        self.0.get(url)
    }
    /// Execute a Rest request in the background.
    ///
    /// # Arguments
    ///
    /// - `request` is what will be sent to the Rest client endpoint.
    ///
    pub fn execute(&self, request: Request) -> RestClientHandle {
        let client = self.0.clone();
        let client_handle = spawn(move || match client.execute(request) {
            Err(err) => RestClientResult::ExecuteError(err.to_string()),
            Ok(response) => match response.status() {
                StatusCode::OK => match response.bytes() {
                    Ok(bytes) => RestClientResult::Body(bytes.into()),
                    Err(err) => RestClientResult::ResponseError(err.to_string()),
                },
                status_code => RestClientResult::HttpStatusCode(status_code.as_u16()),
            },
        });
        // let client_handle = spawn(move || {
        //     thread::sleep(Duration::new(10, 0));
        //     RestClientResult::ExecuteError("Nothing to do".to_string())
        // });
        RestClientHandle::new(client_handle)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use serde_json::Value;
//     use std::time::Duration;
//
//     #[test]
//     fn client() {
//         let rest_client = RestClient::new(Client::default());
//         let mut stopwatch = toolslib::stopwatch::StopWatch::new();
//         let url = Url::parse("https://postman-echo.com/get/with/more/path").unwrap();
//         let request = rest_client.get(url).build().unwrap();
//         let client_handle = rest_client.execute(request);
//         stopwatch.start();
//         let mut wait_count = 0usize;
//         while wait_count < 500 {
//             if client_handle.is_finished() {
//                 stopwatch.stop();
//                 break;
//             }
//             std::thread::sleep(Duration::from_millis(10));
//             wait_count += 1;
//         }
//         assert!(wait_count < 500, "timeout");
//         eprintln!("wait count {}, elapsed {}ms", wait_count, stopwatch.millis());
//         let result = client_handle.get();
//         match result {
//             RestClientResult::Body(body) => {
//                 let document = serde_json::from_slice::<Value>(&body[..]).unwrap();
//                 eprintln!("{}", serde_json::to_string_pretty(&document).unwrap())
//             }
//             result => assert!(false, "{:?}", result),
//         }
//         eprintln!("{:?}", client_handle.get());
//     }
// }
