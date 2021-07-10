use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::sync::atomic::{AtomicUsize, Ordering};


struct AppState {
    app_name: &'static str,
    counter: AtomicUsize,
}

impl AppState {
    pub fn new(app_name: &'static str) -> Self {
        Self {
            app_name,
            counter: AtomicUsize::new(1)
        }
    }
}

#[get("/")]
async fn index<'a>(app_data: web::Data<AppState>) -> impl Responder {
    let counter = app_data.counter.fetch_add(1, Ordering::Relaxed);
    let data = format!("Hello {} you are the {} visitor!", app_data.app_name, counter);
    return HttpResponse::Ok().body(data);
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    return HttpResponse::Ok().body(req_body);
}

async fn manual_hello() -> impl Responder {
    return HttpResponse::Ok().body("Hey There!");
}

static APP_NAME: &'static str = "Actix";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = web::Data::new(AppState::new(APP_NAME));

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(index)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .keep_alive(75)
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
