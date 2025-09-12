use axum::{
    routing::get,
    Router
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use webbrowser;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

   // bind the listener
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();

    // spawn the server into a background task
    let server = axum::serve(listener, app);

    tokio::spawn(async move {
        if let Err(err) = server.await {
            eprintln!("server error: {}", err);
        }
    });

    // open the browser
    if webbrowser::open("http://localhost:3000").is_ok() {
        println!("Opened in browser!");
    }

    // keep the main task alive
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for shutdown signal");
    println!("Shutting down");
}
