use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use regex::Regex;
use reqwest::blocking;

mod query_handler;

pub fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer);
    let request_line = request.lines().next().unwrap_or("");

    if request_line.starts_with("GET ") {
        let path = &request_line[4..request_line.find(" HTTP/").unwrap_or(request_line.len())];

        let response = if path == "/" {
            // Serve index.html
            serve_file("index.html")
        } else if path.starts_with("/search/?query=") {
            // Extract query value
            let query_value = &path["/search/?query=".len()..];

            // Load search.html and replace placeholder
            match fs::read_to_string("frontend/search.html") {
                Ok(contents) => {
                    let urls = query_handler::handle_query(&query_value);
                    let page = build_search_page(urls);
                    format_response(&page, "text/html")
                }
                Err(_) => not_found(),
            }
        } else {
            // Remove leading slash and serve file directly
            let filename = &path[1..];
            serve_file(filename)
        };

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

/// Unified serve_file with MIME detection
fn serve_file(path: &str) -> String {
    let mime = guess_mime_type(path);

    match fs::read_to_string(format!("frontend/{}", path)) {
        Ok(contents) => format_response(&contents, mime),
        Err(_) => not_found(),
    }
}

/// Guess MIME type from extension
fn guess_mime_type(path: &str) -> &str {
    if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".txt") {
        "text/plain"
    } else {
        "application/octet-stream" // fallback for unknown
    }
}

/// Format HTTP response with MIME type
fn format_response(body: &str, mime: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        mime,
        body.len(),
        body
    )
}

/// 404 Not Found
fn not_found() -> String {
    "HTTP/1.1 404 NOT FOUND\r\nContent-Type: text/html\r\n\r\n<h1>404 Not Found</h1>".to_string()
}

/// Fetch <title> from a URL
fn fetch_title(url: &str) -> String {
    if let Ok(resp) = blocking::get(url) {
        if let Ok(body) = resp.text() {
            let re = Regex::new(r"(?i)<title>(.*?)</title>").unwrap();
            if let Some(cap) = re.captures(&body) {
                return cap[1].trim().to_string();
            }
        }
    }
    url.to_string() // fallback to URL if title not found
}

/// Fetch favicon URL from a page
fn fetch_favicon(url: &str) -> String {
    if let Ok(resp) = blocking::get(url) {
        if let Ok(body) = resp.text() {
            let re_icon = Regex::new(r#"(?i)<link[^>]+rel=["']?(?:shortcut icon|icon|apple-touch-icon)["']?[^>]+>"#).unwrap();
            let re_href = Regex::new(r#"href=["']([^"']+)["']"#).unwrap();

            if let Some(icon_tag) = re_icon.captures(&body) {
                if let Some(href_cap) = re_href.captures(&icon_tag[0]) {
                    let favicon_url = href_cap[1].to_string();

                    // Handle relative paths (e.g., "/favicon.ico")
                    if favicon_url.starts_with('/') {
                        if let Ok(parsed) = reqwest::Url::parse(url) {
                            return format!("{}://{}{}", parsed.scheme(), parsed.host_str().unwrap_or(""), favicon_url);
                        }
                    }

                    return favicon_url;
                }
            }
        }
    }

    // Fallback to default /favicon.ico
    if let Ok(parsed) = reqwest::Url::parse(url) {
        return format!("{}://{}/favicon.ico", parsed.scheme(), parsed.host_str().unwrap_or(""));
    }

    String::new() // fallback empty if parsing fails
}

/// Build search results HTML with favicon and title
fn build_search_page(urls: Vec<String>) -> String {
    let template = fs::read_to_string("frontend/search.html").unwrap_or_default();

    let mut links_html = String::new();
    for url in urls {
        let title = fetch_title(&url);
        let favicon = fetch_favicon(&url);

        links_html.push_str(&format!(
            r#"<div class="result">
                <img src="{2}" alt="favicon" width="16" height="16" style="vertical-align:middle; margin-right:4px;">
                <a href="{0}">{1}</a>
              </div>"#,
            url, title, favicon
        ));
    }

    template.replace("<!-- LINKS WILL BE INJECTED HERE -->", &links_html)
}

