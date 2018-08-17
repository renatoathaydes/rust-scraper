use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, Read};

#[derive(Debug)]
struct HttpRequest {
    method_line: MethodLine,
    headers: HashMap<String, String>,
    body: Vec<u8>,
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
            headers.insert(parts[0].to_owned(), parts[1].to_owned());
        } else {
            return Err(format!("Invalid header line (missing ':'): {}", line).to_owned());
        }
    }

    let mut body = Vec::new();
    reader.read_to_end(&mut body)
        .expect("Could not read body");

    Ok(HttpRequest { method_line, headers, body })
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
    if parts.len() != 3 {
        return Err(format!("Invalid method-line (wrong number of parts): {}", line));
    }

    Ok(MethodLine {
        method: parts[0].to_owned(),
        path: parts[1].to_owned(),
        http_version: parts[2].to_owned(),
    })
}