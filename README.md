# hayweb

Web scraping and HTTP plugin for Hayashi.

## Installation

```bash
hay install sheep-farm/hayweb
```

## Usage

```hayashi
import("sheep-farm/hayweb", as=web)

// HTTP GET request
let response = web::http_get("https://example.com", "{}", 30)
print(response)

// HTTP POST request
let post_response = web::http_post("https://api.example.com/data", "{\"key\":\"value\"}", "{}", 30)
print(post_response)

// Extract text from HTML
let html = "<html><body><h1>Hello</h1></body></html>"
let text = web::scrape_text(html)
print(text)

// Extract links from HTML
let links = web::scrape_links(html)
print(links)

// Download file
let success = web::download_file("https://example.com/file.pdf", "file.pdf")
print(success)
```

## Functions

### HTTP Requests
- `http_get(url, headers, timeout)` - HTTP GET request
- `http_post(url, body, headers, timeout)` - HTTP POST request

### Web Scraping
- `scrape_text(html)` - Extract all text from HTML
- `scrape_links(html)` - Extract all links from HTML
- `scrape_images(html)` - Extract all image URLs from HTML
- `scrape_tables(html)` - Extract tables from HTML as JSON

### HTML Parsing
- `html_select(html, selector)` - Select elements using CSS selector
- `html_attr(html, selector, attribute)` - Extract attribute from elements
- `html_text(html, selector)` - Extract text from elements matching selector

### File Operations
- `download_file(url, filepath)` - Download file from URL to local path

## Development

```bash
cargo build --release
cp target/release/libhayweb.so ~/.hay/packages/sheep-farm/hayweb.so
```

## Dependencies

- **reqwest**: HTTP client library
- **scraper**: HTML parsing library
- **select**: CSS selector support
