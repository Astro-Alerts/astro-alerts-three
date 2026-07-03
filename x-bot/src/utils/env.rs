extern crate dotenv;

use std::env;

#[cfg(debug_assertions)]
use dotenv::dotenv;

pub fn init() {
    #[cfg(debug_assertions)]
    dotenv().ok();
}

pub fn _astro_alerts_api_key() -> String {
    env::var("ASTRO_ALERTS_API_KEY").expect("Environment variable ASTRO_ALERTS_API_KEY is required")
}

pub fn api_key() -> String {
    env::var("API_KEY").expect("Environment variable API_KEY is required")
}

pub fn api_key_secret() -> String {
    env::var("API_KEY_SECRET").expect("Environment variable API_KEY_SECRET is required")
}

pub fn _bearer_token() -> String {
    env::var("BEARER_TOKEN").expect("Environment variable BEARER_TOKEN is required")
}

pub fn access_token() -> String {
    env::var("ACCESS_TOKEN").expect("Environment variable ACCESS_TOKEN is required")
}

pub fn access_token_secret() -> String {
    env::var("ACCESS_TOKEN_SECRET").expect("Environment variable ACCESS_TOKEN_SECRET is required")
}
