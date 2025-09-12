use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use webbrowser;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route_service("/", ServeDir::new("dist")
            .not_found_service(ServeFile::new("dist/index.html")
        )
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();

    if webbrowser::open("http://localhost:3000").is_ok() {
        println!("Opened browser at localhost:3000");
    }

    axum::serve(listener, app).await.unwrap();
}
