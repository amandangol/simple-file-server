use std::collections::HashMap;
use std::fmt::Display;
use super::request::Version;

#[derive(Debug)]
pub struct HttpResponse {
    pub version: Version,
    pub status: ResponseStatus,
    pub headers: HashMap<String, String>,
    pub response_body: Vec<u8>,
    pub current_path: String,
}

#[derive(Debug)]
pub enum ResponseStatus {
    OK,
    NotFound,
    BadRequest,
    Forbidden,
    InternalServerError,
}

impl ResponseStatus {
    pub fn code(&self) -> u16 {
        match self {
            ResponseStatus::OK => 200,
            ResponseStatus::NotFound => 404,
            ResponseStatus::BadRequest => 400,
            ResponseStatus::Forbidden => 403,
            ResponseStatus::InternalServerError => 500,
        }
    }

    pub fn reason(&self) -> &str {
        match self {
            ResponseStatus::OK => "OK",
            ResponseStatus::NotFound => "Not Found",
            ResponseStatus::BadRequest => "Bad Request",
            ResponseStatus::Forbidden => "Forbidden",
            ResponseStatus::InternalServerError => "Internal Server Error",
        }
    }
}

impl Display for ResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code(), self.reason())
    }
}

impl HttpResponse {
    pub fn new(version: Version, status: ResponseStatus, current_path: String) -> Self {
        let clean_path = current_path.trim_start_matches('/').to_string();
        let mut response = HttpResponse {
            version,
            status,
            headers: HashMap::new(),
            response_body: Vec::new(),
            current_path: clean_path,
        };
        response.add_header("Accept-Ranges", "bytes");
        response
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_lowercase(), value.to_string());
    }

    pub fn set_body(&mut self, body: impl Into<Vec<u8>>) {
        self.response_body = body.into();
        self.add_header("Content-Length", &self.response_body.len().to_string());
    }

    pub fn to_string(&self) -> Vec<u8> {
        let mut response = format!(
            "{} {}\r\n",
            self.version,
            self.status
        ).into_bytes();

        for (key, value) in &self.headers {
            response.extend(format!("{}: {}\r\n", key, value).into_bytes());
        }

        response.extend(b"\r\n");
        response.extend(&self.response_body);

        response
    }

    pub fn formatted_output(&self) -> String {
        format!(
            "HttpResponse{{ version: {:?}, status: {:?}, content_length: {}, accept_ranges: {}, response_body: <{} bytes>, current_path: {:?} }}",
            self.version,
            self.status,
            self.headers.get("content-length").unwrap_or(&String::from("0")),
            self.headers.get("accept-ranges").unwrap_or(&String::from("none")),
            self.response_body.len(),
            self.current_path,
        )
    }

    pub fn http_response_string(&self) -> String {
        format!(
            "{} {}\naccept-ranges: {}\ncontent-length: {}",
            self.version,
            self.status,
            self.headers.get("accept-ranges").unwrap_or(&String::from("none")),
            self.headers.get("content-length").unwrap_or(&String::from("0"))
        )
    }
}