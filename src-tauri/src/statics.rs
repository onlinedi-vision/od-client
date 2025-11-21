use std::sync::Arc;
use once_cell::sync::Lazy;

pub(crate) static REQ_CLIENT: Lazy<Arc<reqwest::Client>> = Lazy::new(|| {
    Arc::new(reqwest::Client::new())
});
