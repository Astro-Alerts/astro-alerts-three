use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use reqwest::{
    header::{
        HeaderMap,
        ACCEPT,
    },
    Client,
    ClientBuilder,
};
use crate::repository::mongo::MongoRepository;

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(
        ACCEPT,
        "application/json"
            .parse()
            .unwrap(),
    );

    headers
}

pub static MONGO_REPOSITORY: OnceCell<MongoRepository> = OnceCell::new();

lazy_static! {
    pub static ref DEFAULT_CLIENT: Client = ClientBuilder::new()
        .user_agent("AA")
        .default_headers(default_headers())
        .build()
        .expect("reqwest client could not be built");
}
