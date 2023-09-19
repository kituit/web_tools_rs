use std::{path::PathBuf, fs::remove_file};

use axum::{routing::{get, get_service, put, post, delete}, Router, extract::{State, FromRef}, response::{Html, IntoResponse}, Form};
use nanoid::nanoid;
use sqlx::{PgPool, query, FromRow, query_as};
use tera::{Tera, Context};
use tokio::{fs::{File, OpenOptions}, io::{AsyncReadExt, AsyncWriteExt}};
use tower_http::services::ServeFile;

type Db = PgPool;

#[derive(Clone)]
struct AppState {
    tera: Tera,
    db: Db,
    assets_folder: PathBuf
}

impl AppState {
    fn new(tera: Tera, db: Db, assets_folder: PathBuf) -> Self {
        AppState { tera, db, assets_folder }
    }

    fn get_asset_path(&self, asset: &str) -> PathBuf {
        let mut board_path = self.assets_folder.clone();
        board_path.push(asset);
        board_path
    }
}

impl FromRef<AppState> for Tera {
    fn from_ref(input: &AppState) -> Self {
        input.tera.clone()
    }
}

impl FromRef<AppState> for Db {
    fn from_ref(input: &AppState) -> Self {
        input.db.clone()
    }
}

impl FromRef<AppState> for PathBuf {
    fn from_ref(input: &AppState) -> Self {
        input.assets_folder.clone()
    }
}

#[derive(FromRow, Debug)]
struct Url {
    id: i64,
    url: String,
    redirect: String
}

async fn hello_world(State(tera): State<Tera>) -> impl IntoResponse {
    Html(tera.render("hello_world.html", &Context::new()).unwrap())
}

async fn test_db(State(db): State<Db>) -> impl IntoResponse {
    let url = "hello";
    let redirect = nanoid!(10);
    query!("INSERT INTO Urls (url, redirect) VALUES ($1, $2)", url, redirect).execute(&db).await.unwrap();


    let urls = query_as!(Url, "SELECT * FROM urls;").fetch_all(&db).await.unwrap();

    urls.iter().map(|url| format!("{url:?}")).collect::<Vec<String>>().join("||")
}

#[derive(serde::Deserialize)]
struct Board {
    text: String
}

async fn get_board(State(assets_folder): State<PathBuf>, State(tera): State<Tera>) -> impl IntoResponse {
    let mut board_path = assets_folder.clone();
    board_path.push("board.txt");

    let open_result = File::open(board_path).await;

    let text = match open_result {
        Ok(mut file) => {
            let mut buffer = String::new();
            let _ = file.read_to_string(&mut buffer).await.unwrap();
            buffer
        },
        Err(_) => String::new(),
    };


    let mut context = Context::new();
    context.insert("text", &text);
    Html(tera.render("board.html", &context).unwrap())
}

async fn get_board_edit(State(assets_folder): State<PathBuf>, State(tera): State<Tera>) -> impl IntoResponse {
    let mut board_path = assets_folder.clone();
    board_path.push("board.txt");
    let open_result = File::open(board_path).await;

    let text = match open_result {
        Ok(mut file) => {
            let mut buffer = String::new();
            let _ = file.read_to_string(&mut buffer).await.unwrap();
            buffer
        },
        Err(_) => String::new(),
    };

    let mut context = Context::new();
    context.insert("text", &text);
    Html(tera.render("board_edit.html", &context).unwrap())
}

async fn put_board_edit(State(assets_folder): State<PathBuf>, State(tera): State<Tera>, Form(board): Form<Board>) -> impl IntoResponse {
    let mut board_path = assets_folder.clone();
    board_path.push("board.txt");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(board_path)
        .await
        .unwrap();

    let _= file.write_all(board.text.as_bytes()).await.unwrap();

    let mut context = Context::new();
    context.insert("text", &board.text);
    Html(tera.render("board.html", &context).unwrap())
}

async fn delete_board(State(app_state): State<AppState>) -> impl IntoResponse {
    let board_path = app_state.get_asset_path("board.txt");
    let _ = remove_file(board_path);

    let mut context = Context::new();
    context.insert("text", "");
    Html(app_state.tera.render("board.html", &context).unwrap())
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_static_folder::StaticFolder(folder = "templates")] templates_folder: PathBuf,
    #[shuttle_static_folder::StaticFolder(folder = "assets")] assets_folder: PathBuf,
    #[shuttle_aws_rds::Postgres] db: PgPool
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!().run(&db).await.expect("DB ERROR");
    let tera = Tera::new(&format!("{}/**/*", templates_folder.to_str().unwrap())).unwrap();
    let state = AppState::new(tera, db, assets_folder);

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/db", get(test_db))
        .route("/board", get(get_board))
        .route("/board/edit", get(get_board_edit))
        .route("/board/edit", put(put_board_edit))
        .route("/board/delete", delete(delete_board))
        .with_state(state)
        .nest_service("/style.css", get_service(ServeFile::new("templates/style.css")));

    Ok(router.into())
}
