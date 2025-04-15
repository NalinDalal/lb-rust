use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

pub async fn run_backend(port: u16, msg: &'static str) {
    let make_svc = make_service_fn(move |_| {
        let msg = msg.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |_req: Request<Body>| {
                let msg = msg.clone();
                async move {
                    Ok::<_, hyper::Error>(Response::new(Body::from(msg)))
                }
            }))
        }
    });

    let addr = ([127, 0, 0, 1], port).into();
    let server = Server::bind(&addr).serve(make_svc);

    println!("Backend running on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("Backend error: {}", e);
    }
}

