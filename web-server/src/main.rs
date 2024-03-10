use lazy_static::lazy_static;
use log::{error, info};
use simple_logger::SimpleLogger;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

mod router;
mod thread_pool;

lazy_static! {
    static ref ROUTES: router::Router = router::Router::new();
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let _router: &router::Router = &*ROUTES;

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = thread_pool::ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = match stream {
            Err(e) => {
                error!("Failed to establish a connection: {:#?}", e);
                continue;
            }
            Ok(stream) => stream,
        };

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let router: &router::Router = &*ROUTES;

    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    info!("Request: {:#?}", request_line);

    let mut split_iter = request_line.split_whitespace();
    let _method = split_iter.next().unwrap();
    let path = split_iter.next().unwrap();

    let response = match router.get_routes().get(path) {
        Some(_) => {
            let route_data = router.get_routes().get(path).unwrap();
            handle_route(route_data)
        }
        None => {
            error!("Route not found: {:#?}", path);
            String::from("HTTP/1.1 404 NOT FOUND\r\n\r\n")
        }
    };

    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_route(path: &String) -> String {
    let contents = std::fs::read_to_string(path).unwrap();
    let status_line = "HTTP/1.1 200 OK";
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    info!("Response: {:#?}, File: {:#?}", status_line, path);
    response
}
