mod backend;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Uri};
use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        backend::run_backend(8081, "Hello from Backend Server 1").await;
    });
    tokio::spawn(async {
        backend::run_backend(8082, "Hello from Backend 2").await;
    });

    // Wait a bit for the backends to start
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // Backend list & shared counter
    let backends = Arc::new(vec![
        "http://127.0.0.1:8081".to_string(),
        "http://127.0.0.1:8082".to_string(),
    ]);
    let counter = Arc::new(AtomicUsize::new(0));
    let make_svc = make_service_fn(move |_conn| {
        let backends = Arc::clone(&backends);
        let counter = Arc::clone(&counter);
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let backends = Arc::clone(&backends);
                let counter = Arc::clone(&counter);
                async move { handle_request(req, backends, counter).await }
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 8080).into();
    println!("Load Balancer running on http://{}", addr);
    if let Err(e) = Server::bind(&addr).serve(make_svc).await {
        eprintln!("LB error: {}", e);
    }
}

async fn handle_request(
    req: Request<Body>,
    backends: Arc<Vec<String>>,
    counter: Arc<AtomicUsize>,
) -> Result<Response<Body>, hyper::Error> {
    // Pick backend in round-robin
    let index = counter.fetch_add(1, Ordering::SeqCst) % backends.len();
    let selected = &backends[index];

    // Build target URI
    let uri_string = format!(
        "{}{}",
        selected,
        req.uri()
            .path_and_query()
            .map(|x| x.as_str())
            .unwrap_or("/")
    );
    let uri: Uri = uri_string.parse().unwrap();

    // Forward the request
    let client = Client::new();
    let new_req = Request::builder()
        .method(req.method())
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    client.request(new_req).await
}
