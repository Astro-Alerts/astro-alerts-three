use actix_web::dev::Server;
use actix_web::{middleware, web::Data, App, HttpServer};
use deadpool_redis::{Config, Pool};
use std::sync::Arc;
use mongodb::options::ClientOptions;

mod api;
mod core;
mod utils;
mod repository;
mod model;

use core::data::update_data;
use core::time_left::check_time_left_to_launch;
use core::clean::clean_launch_data;
use repository::mongo::MongoRepository;
use utils::constants::MONGO_REPOSITORY;

use api::launch_data::list_launch_data;
use api::misc::ping;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "debug");
    // std::env::set_var("RUST_BACKTRACE", "1");

    utils::env::init();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting api server");

    let client_options: ClientOptions = ClientOptions::parse(utils::env::mongo_uri())
        .await
        .expect("Could not parse database connection client options");

    let _ = MONGO_REPOSITORY.set(MongoRepository::init(client_options.clone()));

    // let redis_client = Client::open("redis://127.0.0.1/").expect("Failed to create Redis Client");
    // let mut redis_connection = redis_client.get_connection().expect("Failed to create Redis Connection");

    let redis_cfg = Config::from_url("redis://127.0.0.1/");
    let redis_pool = redis_cfg.create_pool(None).expect("Failed to create Redis Pool");
    let redis_pool_arc = std::sync::Arc::new(redis_pool);

    log::info!("redis connection online");

    let redis_pool_launch_data = Arc::clone(&redis_pool_arc);
    tokio::spawn(async move {
        let mut interval: tokio::time::Interval =
            tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 Minutes

        interval.tick().await;

        loop {
            interval.tick().await;
            update_data(&redis_pool_launch_data).await;
        }
    });

    let redis_pool_launch_time = Arc::clone(&redis_pool_arc);
    tokio::spawn(async move {
        let mut interval: tokio::time::Interval =
            tokio::time::interval(tokio::time::Duration::from_secs(60)); // 1 Minute

        interval.tick().await;

        loop {
            interval.tick().await;
            check_time_left_to_launch(&redis_pool_launch_time).await;
        }
    });

    tokio::spawn(async move {
        let mut interval: tokio::time::Interval =
            tokio::time::interval(tokio::time::Duration::from_secs(3600)); // 1 Hour

        interval.tick().await;

        loop {
            interval.tick().await;
            clean_launch_data().await;
        }
    });

    let server: Server = HttpServer::new(move || {
        let mongo_data: Data<&MongoRepository> = Data::new(MONGO_REPOSITORY.get().unwrap());

        App::new()
            .wrap(middleware::Logger::default())
            .app_data(mongo_data)
            .service(list_launch_data)
            .service(ping)
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    log::info!("api server is online");

    server.await
}
