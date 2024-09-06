use std::{collections::HashMap, fmt::Display, str::FromStr};

#[derive(Debug)]
pub struct HttpRequest {
    method: Method,
    route: Route,
    version: Version,
    headers: HashMap<String, String>,
    request_body: String,
}

impl HttpRequest {
    pub fn new(request: &str) -> Option<HttpRequest> {
        let lines: Vec<&str> = request.lines().collect();
        let first_line = lines.first()?;
        let parts: Vec<&str> = first_line.split_whitespace().collect();

        if parts.len() != 3 {
            return None;
        }

        let method = Method::from_str(parts[0]).ok()?;
        let route = Route::new(parts[1]);
        let version = Version::from_str(parts[2]).ok()?;
        let headers = HttpRequest::parse_headers(request)?;

        let body_start = request.find("\r\n\r\n").map(|i| i + 4).unwrap_or(request.len());
        let request_body = request[body_start..].to_string();

        Some(HttpRequest {
            method,
            route,
            version,
            headers,
            request_body,
        })
    }

    fn parse_headers(request: &str) -> Option<HashMap<String, String>> {
        let mut headers = HashMap::new();
        let (_, header_str) = request.split_once("\r\n")?;

        for line in header_str.split_terminator("\r\n") {
            if line.is_empty() {
                break;
            }
            let (header, value) = line.split_once(":")?;
            headers.insert(header.trim().to_string(), value.trim().to_string());
        }

        Some(headers)
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &str {
        &self.request_body
    }
}

#[derive(Debug)]
pub struct HttpHeader {
    headers: HashMap<String, String>,
}

impl HttpHeader {
    pub fn new(request: &str) -> Option<HttpHeader> {
        let mut headers = HashMap::new();
        let (_, header_str) = request.split_once("\r\n")?;

        for line in header_str.split_terminator("\r\n") {
            if line.is_empty() {
                break;
            }
            let (header, value) = line.split_once(":")?;
            headers.insert(header.trim().to_string(), value.trim().to_string());
        }

        Some(HttpHeader { headers })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Version {
    V1_1,
    V2_0,
}

impl FromStr for Version {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.1" => Ok(Version::V1_1),
            "HTTP/2.0" => Ok(Version::V2_0),
            _ => Err(()),
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::V1_1 => write!(f, "HTTP/1.1"),
            Version::V2_0 => write!(f, "HTTP/2.0"),
        }
    }
}

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Trace,
    Connect,
    Patch,
}

impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "HEAD" => Ok(Method::Head),
            "OPTIONS" => Ok(Method::Options),
            "TRACE" => Ok(Method::Trace),
            "CONNECT" => Ok(Method::Connect),
            "PATCH" => Ok(Method::Patch),
            _ => Err(()),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
            Method::Head => write!(f, "HEAD"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Trace => write!(f, "TRACE"),
            Method::Connect => write!(f, "CONNECT"),
            Method::Patch => write!(f, "PATCH"),
        }
    }
}

#[derive(Debug)]
pub struct Route {
    path: String,
}

impl Route {
    pub fn new(path: &str) -> Self {
        // Remove leading '/' if present
        let clean_path = path.trim_start_matches('/').to_string();
        Route { path: clean_path }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}