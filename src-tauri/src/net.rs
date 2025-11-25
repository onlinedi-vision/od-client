use crate::statics::REQ_CLIENT;
use std::collections::HashMap;
use reqwest::{Response, Error};
use std::future::Future;

#[inline(always)]
pub(crate) fn post(
    url: &str,
    payload: HashMap<&str, std::string::String>
) -> impl Future<Output = Result<Response, Error>> {
    REQ_CLIENT.clone()
        .post(url)
        .json(&payload)
        .send()
}

#[inline(always)]
pub(crate) async fn post_to_text(
    url: &str,
    payload: HashMap<&str, std::string::String>,
    error_message: &str
) -> Result<String, Error> {
    post(url, payload)
        .await
        .expect(error_message)
        .text()
        .await
}

#[inline(always)]
fn get(url: &str) -> impl Future<Output = Result<Response, Error>> {
    REQ_CLIENT.clone()
        .get(url)
        .send()
}

#[inline(always)]
pub(crate) async fn get_to_text(
    url: &str,
    error_message: &str
) -> Result<String, Error> {
    get(url)
        .await
        .expect(error_message)
        .text()
        .await
}
