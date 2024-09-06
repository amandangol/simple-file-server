use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream, SocketAddr, Ipv4Addr},
    fs,
    path::{Path, PathBuf},
    env,
};
use crate::http::request::{HttpRequest, Method, Version};
use crate::http::response::{HttpResponse, ResponseStatus};
use url_escape::decode;
use infer;

mod http {
    pub mod request;
    pub mod response;
}

fn create_socket() -> SocketAddr {
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 5500)
}

fn handle_client(mut stream: TcpStream, root_dir: &Path) -> io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let request_str = String::from_utf8_lossy(&buffer);
    println!("Received request:\n{}", request_str);

    match HttpRequest::new(&request_str) {
        Some(request) => {
            println!("Parsed request: {:?}", request);
            let response = match request.method() {
                Method::Get => handle_get_request(&request, root_dir),
                Method::Post => handle_post_request(&request),
                _ => {
                    println!("Unsupported method: {:?}", request.method());
                    HttpResponse::new(request.version().clone(), ResponseStatus::BadRequest, request.route().path().to_string())
                },
            };

            println!("Response: {}", response.formatted_output());
            println!("\n{}", response.http_response_string());
            
            stream.write_all(&response.to_string())?;
        }
        None => {
            println!("Failed to parse request");
            let error_response = HttpResponse::new(Version::V1_1, ResponseStatus::BadRequest, String::from("Invalid Request"));
            stream.write_all(&error_response.to_string())?;
        }
    }

    stream.flush()?;
    Ok(())
}

fn handle_get_request(request: &HttpRequest, root_dir: &Path) -> HttpResponse {
    let decoded_path = decode(request.route().path());
    let requested_path = root_dir.join(decoded_path.trim_start_matches('/'));

    println!("Root dir: {:?}", root_dir);
    println!("Requested path: {:?}", requested_path);

    match is_safe_path(root_dir, &requested_path) {
        Ok(true) => {
            if requested_path.is_dir() {
                println!("Serving directory: {:?}", requested_path);
                handle_directory_listing(request, &requested_path)
            } else if requested_path.is_file() {
                println!("Serving file: {:?}", requested_path);
                handle_file_request(request, &requested_path)
            } else {
                println!("Path not found: {:?}", requested_path);
                HttpResponse::new(request.version().clone(), ResponseStatus::NotFound, "Not Found".to_string())
            }
        },
        Ok(false) => {
            println!("Unsafe path access attempted: {:?}", requested_path);
            HttpResponse::new(request.version().clone(), ResponseStatus::Forbidden, "Forbidden".to_string())
        },
        Err(e) => {
            println!("Error checking path safety: {}", e);
            HttpResponse::new(request.version().clone(), ResponseStatus::NotFound, "Not Found".to_string())
        }
    }
}

fn is_safe_path(root_dir: &Path, requested_path: &Path) -> io::Result<bool> {
    let canonicalized_root = root_dir.canonicalize()?;
    let requested_path_buf = requested_path.to_path_buf();
    
    // Check if the requested path exists
    if !requested_path_buf.exists() {
        // If it doesn't exist, check if its parent directory is within the root
        if let Some(parent) = requested_path_buf.parent() {
            let canonicalized_parent = parent.canonicalize()?;
            return Ok(canonicalized_parent.starts_with(&canonicalized_root));
        }
    } else {
        let canonicalized_requested = requested_path_buf.canonicalize()?;
        return Ok(canonicalized_requested.starts_with(&canonicalized_root));
    }
    
    Ok(false)
}


fn handle_directory_listing(request: &HttpRequest, dir_path: &Path) -> HttpResponse {
    let mut response = HttpResponse::new(request.version().clone(), ResponseStatus::OK, dir_path.to_string_lossy().into_owned());
    let mut content = String::new();
    content.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Directory listing</title>
    <style>
    /* Insert the CSS code here */
    body {
        font-family: Arial, sans-serif;
        line-height: 1.6;
        color: #333;
        max-width: 800px;
        margin: 0 auto;
        padding: 20px;
        background-color: #f4f4f4;
    }
    h1 {
        color: #2c3e50;
        border-bottom: 2px solid #3498db;
        padding-bottom: 10px;
    }
    ul {
        list-style-type: none;
        padding: 0;
    }
    li {
        margin-bottom: 10px;
        background-color: #fff;
        border-radius: 4px;
        overflow: hidden;
    }
    li a {
        display: block;
        padding: 10px 15px;
        color: #2980b9;
        text-decoration: none;
        transition: background-color 0.3s ease;
    }
    li a:hover {
        background-color: #ecf0f1;
    }
    .parent-dir {
        font-weight: bold;
    }
    .file-icon, .folder-icon {
        margin-right: 10px;
    }
    .file-icon::before {
        content: "📄";
    }
    .folder-icon::before {
        content: "📁";
    }
    </style>
</head>
<body>"#);
    content.push_str(&format!("<h1>Directory listing for {}</h1>", dir_path.to_string_lossy()));
    content.push_str("<ul>");

    if let Some(parent) = dir_path.parent() {
        content.push_str(&format!(r#"<li><a href="{}" class="parent-dir"><span class="folder-icon"></span>Parent Directory</a></li>"#, parent.to_string_lossy()));
    }

    match fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    let link = format!("{}", name);
                    let icon_class = if path.is_dir() { "folder-icon" } else { "file-icon" };
                    content.push_str(&format!(r#"<li><a href="{}"><span class="{}"></span>{}</a></li>"#, link, icon_class, name));
                }
            }
        },
        Err(e) => {
            match e.kind() {
                io::ErrorKind::PermissionDenied => {
                    response.status = ResponseStatus::Forbidden;
                    content.push_str(&format!("<p>Access denied: {}</p>", e));
                },
                _ => {
                    response.status = ResponseStatus::InternalServerError;
                    content.push_str(&format!("<p>An error occurred: {}</p>", e));
                }
            }
        }
    }

    content.push_str("</ul></body></html>");

    response.add_header("Content-Type", "text/html");
    response.set_body(content.into_bytes());
    response
}
fn handle_file_request(request: &HttpRequest, file_path: &Path) -> HttpResponse {
    let mut response = HttpResponse::new(request.version().clone(), ResponseStatus::OK, file_path.to_string_lossy().into_owned());

    match fs::read(file_path) {
        Ok(contents) => {
            let content_type = infer::get(&contents)
                .map(|t| t.mime_type().to_string())
                .unwrap_or_else(|| {
                    // Fallback to common MIME types based on file extension
                    match file_path.extension().and_then(|e| e.to_str()) {
                        Some("html") | Some("htm") => "text/html",
                        Some("css") => "text/css",
                        Some("js") => "application/javascript",
                        Some("json") => "application/json",
                        Some("txt") => "text/plain",
                        Some("png") => "image/png",
                        Some("jpg") | Some("jpeg") => "image/jpeg",
                        Some("gif") => "image/gif",
                        Some("svg") => "image/svg+xml",
                        Some("pdf") => "application/pdf",
                        Some("mp4") => "video/mp4",
                        Some("webm") => "video/webm",
                        Some("ogg") => "video/ogg",
                        _ => "application/octet-stream",
                    }.to_string()
                });

            response.add_header("Content-Type", &content_type);
            response.set_body(contents);
        },
        Err(e) => {
            match e.kind() {
                io::ErrorKind::PermissionDenied => {
                    response.status = ResponseStatus::Forbidden;
                    response.set_body(format!("Access denied: {}", e).into_bytes());
                },
                io::ErrorKind::NotFound => {
                    response.status = ResponseStatus::NotFound;
                    response.set_body(b"File not found".to_vec());
                },
                _ => {
                    response.status = ResponseStatus::InternalServerError;
                    response.set_body(format!("An error occurred: {}", e).into_bytes());
                }
            }
        }
    }
    response
}


fn handle_post_request(request: &HttpRequest) -> HttpResponse {
    let mut response = HttpResponse::new(request.version().clone(), ResponseStatus::OK, request.route().path().to_string());
    let body = format!(
        "<html><body><h1>Received POST request</h1><p>Body: {}</p></body></html>",
        request.body()
    );
    response.add_header("Content-Type", "text/html");
    response.set_body(body);
    response
}

fn serve(socket: SocketAddr, root_dir: PathBuf) -> io::Result<()> {
    let listener = TcpListener::bind(socket)?;
    println!("Server listening on {} serving directory {:?}", socket, root_dir);
    
    for stream in listener.incoming() {
        let stream = stream?;
        let root_dir = root_dir.clone();
        if let Err(e) = handle_client(stream, &root_dir) {
            eprintln!("Error handling client: {}", e);
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let root_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        env::current_dir()?
    };

    let socket = create_socket();
    serve(socket, root_dir)
}