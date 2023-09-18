use std::path::PathBuf;

use axum::{routing::get, Router, extract::State, response::{Html, IntoResponse}};
use tera::{Tera, Context};

async fn hello_world(State(tera): State<Tera>) -> impl IntoResponse {
    Html(tera.render("hello_world.html", &Context::new()).unwrap())
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_static_folder::StaticFolder(folder = "templates")] templates_folder: PathBuf
) -> shuttle_axum::ShuttleAxum {
    let tera = Tera::new(&format!("{}/**/*", templates_folder.to_str().unwrap())).unwrap();
    let router = Router::new()
        .route("/", get(hello_world))
        .with_state(tera);

    Ok(router.into())
}
