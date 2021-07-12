use actix_web::{get, post, web, App, HttpRequest, HttpResponse, dev::HttpResponseBuilder, HttpServer, Responder, error, Error, http::StatusCode, http::header};
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::future::{ready, Ready};
use serde::Serialize;
use serde_json::to_string;
use std::collections::HashMap;
use std::sync::RwLock;
use derive_more::{Display, Error};


struct AppState {
    app_name: &'static str,
    counter: AtomicUsize,
    users: RwLock<HashMap<u64, User>>,
}

impl AppState {
    pub fn new(app_name: &'static str) -> Self {
        Self {
            app_name,
            counter: AtomicUsize::new(1),
            users: RwLock::new(HashMap::new()),
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

#[derive(Serialize, Clone)]
struct User {
    name: String,
    id: u64,
}

impl Responder for User {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = to_string(&self).unwrap();

        return ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)));
    }
}

#[derive(Debug, Display, Error)]
enum UserError {
    #[display(fmt = "user does not exist")]
    UserDoesNotExist,
}

impl error::ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        return HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        return match *self {
            UserError::UserDoesNotExist => StatusCode::NOT_FOUND,
        };
    }
}

#[get("/user/{user_id}")]
async fn get_user(app_data: web::Data<AppState>, web::Path(user_id): web::Path<u64>) -> Result<User, UserError> {
    let users = app_data.users.read().unwrap();
    return match users.get(&user_id) {
        Some(user) => { Ok((*user).clone()) },
        None => { Err(UserError::UserDoesNotExist) }
    };
}

#[post("/user/{user_id}/{name}")]
async fn post_user(app_data: web::Data<AppState>, web::Path((user_id, name)): web::Path<(u64, String)>) -> impl Responder {
    app_data.users.write().unwrap().insert(user_id, User { name: name.clone(), id: user_id });
    return User{ name: name, id: user_id };
}

static APP_NAME: &'static str = "Actix";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Creating the ARC application data prior to creating the server is required for shared
    // application state, otherwise it will create two instances, #docs.
    let app_data = web::Data::new(AppState::new(APP_NAME));

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone()) // This is cloning the ARC, not the AppState object.
            .service(index)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            .service(get_user)
            .service(post_user)
    })
    .keep_alive(75)
    .bind("127.0.0.1:9090")?
    .run()
    .await
}
