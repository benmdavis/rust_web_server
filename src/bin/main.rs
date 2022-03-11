use std::io;
use std::env;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::*;
use server::ThreadPool;
use chrono::{DateTime, Local};
use bufstream::BufStream;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap(); //TOO build error handler
        pool.execute(|| {
            handle_connection(stream).unwrap();
        });
    }
}

fn handle_connection(stream: TcpStream)  -> io::Result<()> {
    let mut buffer = BufStream::new(stream);
    
    // stream.read(&mut buffer).unwrap();
    let mut request_line = String::new();

    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    
    // let (status_line, filename) = if buffer.starts_with(get) {
    //     ("HTTP/1.1 200 OK", "content/index.html")
    // } else {
    //     ("HTTP/1.1 404 NOT FOUND", "content/404.html")
    // };
    buffer.read_line(&mut request_line).unwrap();
    
    match parse_request(&mut request_line) {
        Ok(request) => {
            log_request(&request);
            let response = handle_request(&request);
            buffer.write(response.as_bytes());
        }
        Err(()) => {
            println!("Bad request: {}", &request_line)
        },
    }

    Ok(())
 
}

struct Request {
    http_version: String,
    method: String,
    path: String,
    time: DateTime<Local>,
}

fn parse_request(request: &mut String) -> Result<Request, ()> {
    let mut parts = request.split(" ");
    let method = match parts.next() {
        Some(method) => method.trim().to_string(),
        None => return Err(()),
    };
    
    let path = match parts.next() {
        Some(path) => format!("./content{}", path.trim().to_string()),
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

fn handle_request(request: &Request) -> String {
    println!("{}", request.path);

    let content = std::fs::read_to_string(request.path.clone());
    // let temp = content.clone();

    // println!("{:?}", content);

    let status_line = match content {
        Ok(_) => "HTTP/1.1 200 OK",
        Err(_) => "HTTP/1.1 404 NOT FOUND",   
    };

    let content = match content {
        Ok(content) => content,
        Err(_) => std::fs::read_to_string("content/404.html").unwrap(),
    };
    
    // let (status_line, filename) = if buffer.starts_with(get) {
    //     ("HTTP/1.1 200 OK", "content/index.html")
    
    let response = format!("{}\r\nContent-Length:{}\r\n\r\n{}", status_line, content.len(), content);
    
    return response;
    // stream.write(response.as_bytes()).unwrap();
    // stream.flush().unwrap();
}
//TODO: Pass request to handle_connection or something. 
//https://concisecoder.io/2019/05/11/creating-a-static-http-server-with-rust-part-1/
