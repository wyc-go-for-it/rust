mod pool;
use pool::ThreadPool;

use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread, time,
};

use chrono::Local;

fn main() {
    use std::os::windows::io::{AsRawSocket, FromRawSocket, IntoRawSocket, RawSocket};
    let listener = TcpListener::bind("127.0.0.1:6868").unwrap();
    let pool: ThreadPool = ThreadPool::new(4);
    for stream in listener.incoming() {
        pool.execute(|| {
            handle_connect(stream.unwrap());
        });
    }

    println!("server has exited");
}

fn handle_connect(mut s: TcpStream) {
    let buf_read = BufReader::new(&mut s);
    let _http_request: Vec<_> = buf_read
        .lines()
        .map(|r| r.unwrap())
        .take_while(|lines| !lines.is_empty())
        .collect();

    if _http_request.is_empty() {
        return;
    }

    let get: Vec<&str> = _http_request[0]
        .split(" ")
        .filter(|s| s.starts_with("/"))
        .collect();

    let status_line;
    let content;
    let len;

    if get.is_empty() {
        status_line = "HTTP/1.1 404 Not found";
        content = not_found("");
        len = content.len();
    } else {
        let f = get[0];
        let file = format!(".{f}");
        match fs::read_to_string(file) {
            Ok(_) => {
                status_line = "HTTP/1.1 200 OK";
                content = response();
                len = content.len();

                thread::sleep(time::Duration::from_secs(5));
            }
            Err(_) => {
                status_line = "HTTP/1.1 404 Not found";
                content = not_found(f);
                len = content.len();
            }
        }
    }

    s.write_all(format!("{status_line}\r\nContent-Length:{len}\r\n\r\n{content}").as_bytes())
        .unwrap_or_default();
}

fn not_found(file: &str) -> String {
    format!(
        "<!DOCTYPE html>
    <html lang=\"en\">
      <head>
        <meta charset=\"utf-8\">
        <title>你好!</title>
      </head>
      <body>
        <h1>很抱歉!</h1>
        <p>{file} 未找到</p>
      </body>
    </html>"
    )
}

fn response() -> String {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    format!(
        "<!DOCTYPE html>
    <html lang=\"en\">
      <head>
        <meta charset=\"utf-8\">
        <title>Hello!</title>
      </head>
      <body>
        <h1>Hello!</h1>
        <p>Hi from Rust<{now}></p>
      </body>
    </html>"
    )
}
