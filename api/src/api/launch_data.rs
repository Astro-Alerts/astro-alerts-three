use actix_web::{get, HttpResponse, HttpRequest, web::Data};
use crate::repository::mongo::MongoRepository;
use crate::utils::env;

#[get("/api/v1/launch_data")]
pub async fn list_launch_data(req: HttpRequest, mongo_repo: Data<&MongoRepository>) -> HttpResponse {
    if !env::api_key_check(req.headers(), env::api_key()).await {
        return HttpResponse::Unauthorized().body("Invalid API_KEY provided.");
    }

    let launch_data = mongo_repo.list_launch_data().await;
    match launch_data {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch all launch data.")
    }
}
