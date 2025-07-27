use std::net::TcpListener;

mod handler;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening on 127.0.0.1:8080");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handler::handle_client(stream);
    }
}
