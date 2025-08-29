//use libmdns::{Service, ServiceDiscovery, ServiceRegistration};
use tiny_http::{Server, Response};
//use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;

fn main() {
    // Start HTTP server on any available port
    let server = Server::http("0.0.0.0:0").unwrap();
    let addr = server.server_addr();
    println!("Server running on http://{}", addr);

    // Spawn server in background
    thread::spawn(move || {
        for request in server.incoming_requests() {
            println!("Received request: {}", request.url());
            let response = Response::from_string("Hello from Rust server!");
            let _ = request.respond(response);
        }
    });

    // mDNS service registration
    let responder = libmdns::Responder::new().unwrap();
    let _svc = responder.register(
        "_http._tcp".to_owned(),
        "libmdns Web Server".to_owned(),
        80,
        &["path=/"],
    );

    /*
    let instance_name = format!("rust-server-{}", addr.port());
    let _svc = Service::new(
        &instance_name,
        "_myapp._tcp",   // Service type
        addr.port(),
        &["version=1.0"],
    )
    .expect("Failed to register mDNS service");
    */

    //println!("mDNS service registered as {} on port {}", instance_name, addr.port());

    // Keep the main thread alive
    loop {
        std::thread::park();
    }
}
