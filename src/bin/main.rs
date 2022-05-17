use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use server::ThreadPool;
use chrono::{DateTime, Local};
use bufstream::BufStream;


struct Request {
    http_version: String,
    method: String,
    path: String,
    time: DateTime<Local>,
}

struct Response {
    response: String,
    content: Vec<u8>,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap(); 
        pool.execute(|| {
            handle_connection(stream).unwrap();
        });
    }
}

fn handle_connection(stream: TcpStream)  -> io::Result<()> {
    let mut buffer = BufStream::new(stream);
    let mut request_line = String::new();
    buffer.read_line(&mut request_line).unwrap();
    
    match parse_request(&mut request_line) {
        Ok(request) => {
            log_request(&request);
            let response = handle_request(&request);
            match buffer.write(response.response.as_bytes()) {
                Ok(_) => println!("Response success"),
                Err(_) => println!("Response failed")
            }
            match buffer.write(&response.content) {
                Ok(_) => println!("Response success"),
                Err(_) => println!("Response failed")
            }
        }
        Err(()) => {
            println!("Bad request: {}", &request_line)
        },
    }

    Ok(())
 
}

fn parse_request(request: &mut String) -> Result<Request, ()> {
    let mut parts = request.split(" ");
     
    let method = match parts.next() {
        Some(method) => method.trim().to_string(),
        None => return Err(()),
    };
    
    let path = match parts.next() {
        Some(path) => format!("././content{}", path.trim().to_string()),
        None => return Err(()),
    };
    let http_version = match parts.next() {
        Some(http_version) => http_version.trim().to_string(),
        None => return Err(()),
    };
    let time = Local::now();

    Ok( Request {
        http_version: http_version,
        method: method,
        path: path,
        time: time
    })
}

fn log_request(request: &Request) {
    println!(
        "[{}] \"{} {} {}\"",
        request.time,
        request.method,
        request.path,
        request.http_version,
    );
}


fn handle_request(request: &Request) -> Response {
    println!("{}", request.path);

    let content = std::fs::read(request.path.clone());

    let status_line = match content {
        Ok(_) => "HTTP/1.1 200 OK",
        Err(_) => "HTTP/1.1 404 NOT FOUND",   
    };

    let content = match content {
        Ok(content) => content,
        Err(_) => {
            println!("{}", content.unwrap_err());
            std::fs::read("content/404.html").unwrap()
        },
    };
    // println!("\n{}: {}\n", request.path, content);

    // let response = format!("{}\r\nContent-Length:{}\r\n\r\n{}", status_line, content.len(), content);
    let response = Response {
        response: format!("{}\r\nContent-Length:{}\r\n\r\n", status_line, content.len()),
        content: content
    };

    return response;
}