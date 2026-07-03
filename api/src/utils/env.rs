use actix_web::http::header::HeaderMap;
use std::env;

#[cfg(debug_assertions)]
extern crate dotenv;
use dotenv::dotenv;

pub fn init() {
    #[cfg(debug_assertions)]
    dotenv().ok();
}

pub fn api_key() -> String {
    env::var("API_KEY").expect("Environment variable API_KEY is required")
}

pub fn mongo_uri() -> String {
    env::var("MONGO_URI").expect("Environment variable MONGO_URI is required")
}

pub async fn api_key_check(headers: &HeaderMap, api_key: String) -> bool {
    if let Some(provided_api_key) = headers.get("api_key") {
        match provided_api_key.to_str() {
            Ok(v) => {
                if v == api_key {
                    return true;
                }
            }
            Err(_) => return false,
        }
    }

    false
}
