use actix_web::{head, HttpResponse};

#[head("/health_check")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().body("Astro Alerts API is Online.")
}
