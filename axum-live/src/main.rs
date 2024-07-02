use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use anyhow::Result;
use axum::{Router, Server, routing::get, routing::post, Json, async_trait, TypedHeader, Extension};
use axum::extract::{FromRequest, Path, RequestParts};
use axum::headers::{Authorization, authorization::Bearer};
use axum::http::{StatusCode};
use axum::response::{IntoResponse, Response};
use jsonwebtoken as jwt;

const SECRET: &[u8] = b"deadbeefdeadbeefdeadbeefdeadbeef";
const NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() -> Result<()>{
    let store = TodoStore::default();
    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/post_todo", post(|| async { "Hello, World!" }))
        // 1. 添加共享变量，如果所有的route都需要的话则需要放到最后面，
        // 2. 这里放的只是为了当前的这个json
        // 3. 进到这个地方的时候会复制出一份给这个路由处理
        .route("/list", get(list_handler).layer(Extension(store.clone())))
        .route("/detail", get(detail_handler).layer(Extension(store.clone())))
        .route("/create_todo", post(create_todo_handler).layer(Extension(store.clone())))
        .route("/login", post(login_handler))
        .route("/api/*path", get(path_handler));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);
    Server::bind(&addr).serve(router.into_make_service()).await?;
    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Todo{
    pub id: usize,
    pub user_id: usize,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Default, Clone)]
struct TodoStore{
    items: Arc<RwLock<Vec<Todo>>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Claims{
    pub id: usize,
    pub username: String,
    // jwt的exp是必须的参数， 并且需要指定一个时间戳做为过期时间
    pub exp: usize,
}

// 获取当前时间戳
fn get_epoc() -> usize{
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch.as_secs() as usize
}

// 实现 FromRequest trait这样就可以直接在接口里面使用claims
#[async_trait]
impl <B> FromRequest<B> for Claims where B: Send{
    type Rejection = HttpError;
    async fn from_request(req: &mut RequestParts<B>) -> std::result::Result<Self, Self::Rejection> {
        // 从请求头中获取 Authorization
        let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request(req).await.map_err(|_| HttpError::Unauthorized).unwrap();
        let key = jwt::DecodingKey::from_secret(SECRET);
        let token = jwt::decode::<Claims>(&bearer.token(), &key, &jwt::Validation::default()).map_err(|_| HttpError::Unauthorized).unwrap();
        Ok(token.claims)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum HttpError{
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
}

impl IntoResponse for HttpError{

    fn into_response(self) -> Response {
        match self{
            HttpError::BadRequest => (StatusCode::BAD_REQUEST, "BAD_REQUEST").into_response(),
            HttpError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED").into_response(),
            HttpError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN").into_response(),
            HttpError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND").into_response(),
            HttpError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "InternalServerError").into_response(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct LoginResponse {
    pub token: String,
}

async fn list_handler(Extension(store): Extension<TodoStore>) -> Result<Json<Vec<Todo>>, HttpError>{
    match store.items.read() {
        Ok(items) => Ok(Json(items.clone())),
        Err(_) => Err(HttpError::InternalServerError),
    }
}

async fn detail_handler(claims: Claims, Extension(store): Extension<TodoStore>) -> Result<Json<Todo>, HttpError>{
    let user_id = claims.id;
    match store.items.read() {
        Ok(items) => Ok(Json(items
            .iter()
            .filter(|item| item.user_id == user_id)
            .map(|item| item.clone())
            .collect::<Vec<Todo>>()[0].clone())),
        Err(_) => Err(HttpError::InternalServerError),
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct CreateTodo{
    pub title: String
}


// 这个地方直接使用Json包装，类似于java中的@RequestBody会解析
async fn create_todo_handler(claims: Claims, Json(todo): Json<CreateTodo>, Extension(store): Extension<TodoStore>) -> Result<StatusCode, HttpError>{
    println!("claims: {:?}", claims);
    match store.items.write(){
        Ok(mut items) => {
            items.push(Todo{
                id: get_next_id(),
                user_id: claims.id,
                title: todo.title,
                completed: false,
            });
            Ok(StatusCode::CREATED)
        },
        Err(_) => Err(HttpError::InternalServerError)
    }
}

// 获取下一个id
fn get_next_id() -> usize{
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}


async fn login_handler(Json(login): Json<LoginRequest>) -> Json<LoginResponse>{
    println!("login: {:?}", login);
    // skip db
    let claims = Claims{
        id: 1,
        username: "admin".to_string(),
        exp: get_epoc() + 14 * 24 * 60 * 60,
    };
    let key = jwt::EncodingKey::from_secret(SECRET);
    let token = jwt::encode(&jwt::Header::default(), &claims, &key).unwrap();
    Json(LoginResponse{
        token,
    })
}

async fn path_handler(path: Path<String>) -> Json<String> {
    Json(format!("path: {}", path.0))
}