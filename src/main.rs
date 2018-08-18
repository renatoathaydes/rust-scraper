use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, Read};

#[derive(Debug)]
struct HttpRequestMeta {
    host: String,
    port: u32,
}

#[derive(Debug)]
struct HttpRequest {
    method_line: MethodLine,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    meta: Option<HttpRequestMeta>,
}

#[derive(Debug)]
struct MethodLine {
    method: String,
    path: String,
    http_version: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        println!("Got file {}", args[1]);
        let file = File::open(&args[1])
            .expect("Cannot open the given file");
        match parse_http_request(&file) {
            // TODO send request, parse response
            Ok(request) => println!("{:?}", request),
            Err(e) => {
                eprintln!("{:?}", e);
                std::process::exit(3);
            }
        };
    } else {
        eprintln!("Incorrect number of arguments. Please provide a single argument, \
            the request file to execute.");
        std::process::exit(1);
    }
}

fn parse_http_request(file: &File) -> Result<HttpRequest, String> {
    let mut reader = io::BufReader::new(file);
    let method_line = parse_method_line(&mut reader)?;
    let mut headers = HashMap::new();

    for line in reader.by_ref().lines() {
        let line = line.expect("Unable to read file line");
        if line == "" {
            break;
        }
        let parts: Vec<&str> = line.splitn(2, ":").collect();
        if parts.len() == 2 {
            headers.insert(parts[0].to_owned(), parts[1].trim().to_owned());
        } else {
            return Err(format!("Invalid header line (missing ':'): {}", line).to_owned());
        }
    }

    let mut body = Vec::new();
    reader.read_to_end(&mut body)
        .expect("Could not read body");

    let meta = Option::None;
    Ok(with_meta(HttpRequest { method_line, headers, body, meta }))
}

fn parse_method_line(reader: &mut io::BufReader<&File>) -> Result<MethodLine, String> {
    let mut line = String::new();
    reader.read_line(&mut line)
        .expect("Could not read request method-line");
    if !line.is_empty() {
        let new_line_idx = line.len() - 1;
        line.remove(new_line_idx);
    }
    let parts: Vec<&str> = line.split(" ").collect();
    if parts.len() != 2 && parts.len() != 3 {
        return Err(format!("Invalid method-line (wrong number of parts): {}", line));
    }
    let http_version = if parts.len() == 3 { parts[2] } else { "HTTP/1.1" };

    Ok(MethodLine {
        method: parts[0].to_owned(),
        path: parts[1].to_owned(),
        http_version: http_version.to_owned(),
    })
}

fn with_meta(mut req: HttpRequest) -> HttpRequest {
    let host: String;
    if req.method_line.path.starts_with("http://") {
        let url_minus_protocol: String = req.method_line.path.chars().skip(7).collect();
        let slash_index = url_minus_protocol.find("/");
        let path_only: String;
        if let Some(index) = slash_index {
            host = url_minus_protocol.chars().take(index).collect();
            path_only = url_minus_protocol.chars().skip(index).collect();
        } else {
            host = url_minus_protocol;
            path_only = "/".to_owned();
        }
        req.method_line.path = path_only;
    } else if req.method_line.path.starts_with("https://") {
        panic!("HTTPS is not supported yet");
    } else {
        host = req.headers.get("Host")
            .expect("Unable to identify Host either from headers or method-line")
            .to_owned();
    }

    let port: u32;
    let fixed_host: String;
    if host.contains(":") {
        let parts: Vec<&str> = host.splitn(2, ":").collect();
        fixed_host = parts[0].to_owned();
        port = parts[1].parse::<u32>().expect("Invalid port")
    } else {
        fixed_host = host.clone();
        port = 80;
    };
    req.headers.insert("Host".to_owned(), fixed_host.clone());
    req.meta = Option::Some(HttpRequestMeta { host: fixed_host, port });
    req
}
