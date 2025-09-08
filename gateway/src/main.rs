use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Định nghĩa router
    let app = Router::new().route("/", get(hello_world));

    // Chạy server tại 0.0.0.0:3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Server running at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}

// Handler
async fn hello_world() -> &'static str {
    "Hello world!"
}