use std::process::exit;
use std::process::Command;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let mut state = State {
        repo: "/".to_string(),
    };

    let listener = match TcpListener::bind("0.0.0.0:6969") {
        Ok(listener) => listener,
        Err(err) => {
            println!("[ERROR]: Unable to start server: {err}");
            exit(1);
        },
    };

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                println!("[ERROR]: Bad connection: {err}");
                continue;
            }
        };
        handle_connection(stream, &mut state);
    }
}

struct State {
    repo: String,
}

fn handle_connection(mut stream: TcpStream, state: &mut State) {
    println!("[INFO]: New Connection");
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    println!("[INFO]: Request {:#?}", http_request);

    let request_line = &http_request[0];
    let command = &http_request[1];
    let (status_line, contents) = if request_line == "GET / HTTP/1.1" {
        process_command(command, state)
    } else {
        println!("[ERROR]: File not found");
        ("HTTP/1.1 404 NOT FOUND".to_string(), "404".to_string())
    };

    let length = contents.len();
    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    );
    stream.write_all(response.as_bytes()).unwrap();
}

const OK: &str = "HTTP/1.1 200 OK";
const BAD_REQUEST: &str = "HTTP/1.1 400 BAD REQUEST";

fn process_command(command: &str, state: &mut State) -> (String, String) {
    let mut iter = command.split_ascii_whitespace();
    if Some("command:") == iter.next() {
        let command = match iter.next() {
            Some(command) => command,
            None => {
                println!("[ERROR]: Bad request");
                return (BAD_REQUEST.to_string(), "400".to_string());
            },
        };
        println!("[INFO]: Requested {command}");
        match command {
            "git-pull" => {
                let result = Command::new("git")
                    .arg("-C")
                    .arg(&state.repo)
                    .arg("pull")
                    .output();
                let output = match result {
                    Ok(output) => output,
                    Err(err) => {
                        let err_str = format!("[ERROR]: Failed to pull: {err}");
                        println!("{}", err_str);
                        return (OK.to_string(), err_str);
                    },
                };
                println!("[INFO]: git -C {repo} pull\n{}", String::from_utf8(output.stdout).unwrap(), repo = &state.repo);
                
            },
            "git-repo" => {
                return (OK.to_string(), format!("Repo: {}", state.repo));
            },
            "set-repo" => {
                let path = match iter.next() {
                    Some(path) => path,
                    None => {
                        println!("[ERROR]: Missing path");
                        return (OK.to_string(), "Missing path".to_string());
                    },
                };
                state.repo = path.to_string();
            },
            _ => {
                return (OK.to_string(), "Unknown command".to_string());
            }
        }
        return (OK.to_string(), format!("{}: OK", command.to_string()));
    }
    return (OK.to_string(), "OK".to_string());
}
