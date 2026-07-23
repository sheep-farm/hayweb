#![allow(clippy::not_unsafe_ptr_arg_deref)]
use hayashi_plugin_sdk::arrow;
use hayashi_plugin_sdk::arrow::array::Array;
use hayashi_plugin_sdk::{hayashi_fn, hayashi_plugin, HayashiValue};
use reqwest::blocking::Client;
use reqwest::header::HeaderValue;
use std::collections::HashMap;
use std::time::Duration;

hayashi_plugin!();

/// 1. http_get(url, headers, timeout)
/// HTTP GET request
/// url: target URL
/// headers: optional headers as JSON string
/// timeout: timeout in seconds (default 30)
#[hayashi_fn]
pub fn http_get(url: String, headers: String, timeout: i64) -> String {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout.max(1) as u64))
        .build()
        .unwrap_or_else(|_| Client::new());

    let mut request = client.get(&url);

    // Parse headers if provided
    if !headers.is_empty() {
        if let Ok(header_map) = serde_json::from_str::<HashMap<String, String>>(&headers) {
            for (key, value) in &header_map {
                if let Ok(header_value) = HeaderValue::from_str(value) {
                    request = request.header(key, header_value);
                }
            }
        }
    }

    match request.send() {
        Ok(response) => {
            let status = response.status().as_u16();
            let headers_out = response
                .headers()
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("\n");

            match response.text() {
                Ok(body) => {
                    format!(
                        "Status: {}\nHeaders:\n{}\nBody:\n{}",
                        status, headers_out, body
                    )
                }
                Err(e) => format!("Error reading body: {}", e),
            }
        }
        Err(e) => format!("Request failed: {}", e),
    }
}

/// 2. http_post(url, body, headers, timeout)
/// HTTP POST request
/// url: target URL
/// body: request body
/// headers: optional headers as JSON string
/// timeout: timeout in seconds (default 30)
#[hayashi_fn]
pub fn http_post(url: String, body: String, headers: String, timeout: i64) -> String {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout.max(1) as u64))
        .build()
        .unwrap_or_else(|_| Client::new());

    let mut request = client.post(&url).body(body);

    // Parse headers if provided
    if !headers.is_empty() {
        if let Ok(header_map) = serde_json::from_str::<HashMap<String, String>>(&headers) {
            for (key, value) in &header_map {
                if let Ok(header_value) = HeaderValue::from_str(value) {
                    request = request.header(key, header_value);
                }
            }
        }
    }

    match request.send() {
        Ok(response) => {
            let status = response.status().as_u16();
            let headers_out = response
                .headers()
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("\n");

            match response.text() {
                Ok(response_body) => {
                    format!(
                        "Status: {}\nHeaders:\n{}\nBody:\n{}",
                        status, headers_out, response_body
                    )
                }
                Err(e) => format!("Error reading body: {}", e),
            }
        }
        Err(e) => format!("Request failed: {}", e),
    }
}

/// 3. scrape_text(html)
/// Extract all text from HTML
/// html: HTML string
#[hayashi_fn]
pub fn scrape_text(html: String) -> String {
    scrape_text_impl(html)
}

fn scrape_text_impl(html: String) -> String {
    // Simple text extraction: remove tags
    let mut result = String::new();
    let mut in_tag = false;
    let chars = html.chars().peekable();

    for c in chars {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }

    // Clean up whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 4. scrape_links(html)
/// Extract all links from HTML
/// html: HTML string
#[hayashi_fn]
pub fn scrape_links(html: String) -> String {
    scrape_links_impl(html)
}

fn scrape_links_impl(html: String) -> String {
    let mut links = Vec::new();

    // Simple regex-like extraction for href attributes
    let mut chars = html.chars().peekable();
    let mut buffer = String::new();

    while let Some(c) = chars.next() {
        buffer.push(c);

        if buffer.ends_with("href=\"") {
            buffer.clear();
            let mut link = String::new();

            while let Some(&next_c) = chars.peek() {
                if next_c == '"' {
                    chars.next();
                    if !link.is_empty() {
                        links.push(link.clone());
                    }
                    break;
                }
                if let Some(c) = chars.next() {
                    link.push(c);
                }
            }
        }

        if buffer.len() > 100 {
            buffer.clear();
        }
    }

    serde_json::to_string(&links).unwrap_or_else(|_| "[]".to_string())
}

/// 5. scrape_images(html)
/// Extract all image URLs from HTML
/// html: HTML string
#[hayashi_fn]
pub fn scrape_images(html: String) -> String {
    scrape_images_impl(html)
}

fn scrape_images_impl(html: String) -> String {
    let mut images = Vec::new();

    // Simple extraction for src attributes in img tags
    let mut chars = html.chars().peekable();
    let mut buffer = String::new();

    while let Some(c) = chars.next() {
        buffer.push(c);

        if buffer.to_lowercase().ends_with("src=\"") {
            buffer.clear();
            let mut src = String::new();

            while let Some(&next_c) = chars.peek() {
                if next_c == '"' {
                    chars.next();
                    if !src.is_empty() {
                        images.push(src.clone());
                    }
                    break;
                }
                if let Some(c) = chars.next() {
                    src.push(c);
                }
            }
        }

        if buffer.len() > 100 {
            buffer.clear();
        }
    }

    serde_json::to_string(&images).unwrap_or_else(|_| "[]".to_string())
}

/// 6. scrape_tables(html)
/// Extract tables from HTML as JSON
/// html: HTML string
#[hayashi_fn]
pub fn scrape_tables(html: String) -> String {
    let mut tables = Vec::new();

    // Simple table extraction: find <table>...</table> blocks
    let mut in_table = false;
    let mut current_table = String::new();
    let mut depth = 0;

    for c in html.chars() {
        if c == '<' {
            let remaining: String = html
                .chars()
                .skip(html.chars().position(|x| x == c).unwrap_or(0))
                .collect();
            if remaining.starts_with("<table") {
                in_table = true;
                depth = 1;
                current_table.clear();
            } else if remaining.starts_with("</table>") {
                depth -= 1;
                if depth == 0 {
                    in_table = false;
                    if !current_table.is_empty() {
                        tables.push(current_table.clone());
                    }
                }
            } else if in_table && remaining.starts_with("<tr") {
                depth += 1;
            } else if in_table && remaining.starts_with("</tr") {
                depth -= 1;
            }
        }

        if in_table {
            current_table.push(c);
        }
    }

    serde_json::to_string(&tables).unwrap_or_else(|_| "[]".to_string())
}

/// 7. download_file(url, filepath)
/// Download file from URL to local path
/// url: source URL
/// filepath: destination path
#[hayashi_fn]
pub fn download_file(url: String, filepath: String) -> bool {
    let client = Client::new();

    match client.get(&url).send() {
        Ok(response) => {
            if response.status().is_success() {
                match response.bytes() {
                    Ok(bytes) => match std::fs::write(&filepath, bytes) {
                        Ok(_) => true,
                        Err(e) => {
                            eprintln!("Failed to write file: {}", e);
                            false
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read response: {}", e);
                        false
                    }
                }
            } else {
                eprintln!("HTTP error: {}", response.status());
                false
            }
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
            false
        }
    }
}

/// 8. html_select(html, selector)
/// Select elements using CSS selector
/// html: HTML string
/// selector: CSS selector (simplified: tag, .class, #id)
#[hayashi_fn]
pub fn html_select(html: String, selector: String) -> String {
    // Simplified CSS selector implementation
    let mut results = Vec::new();

    if let Some(class_name) = selector.strip_prefix('.') {
        // Class selector
        let search_pattern = format!("class=\"{}\"", class_name);
        let chars = html.chars().peekable();
        let mut buffer = String::new();

        for c in chars {
            buffer.push(c);

            if buffer.contains(&search_pattern) {
                // Extract element content (simplified)
                let start = html.chars().position(|x| x == '<').unwrap_or(0);
                let remaining: String = html.chars().skip(start).collect();

                if let Some(end) = remaining.find('>') {
                    let content_start = start + end + 1;
                    if let Some(content_end) = html[content_start..].find('<') {
                        let content = &html[content_start..content_start + content_end];
                        results.push(content.trim().to_string());
                    }
                }
                buffer.clear();
            }

            if buffer.len() > 500 {
                buffer.clear();
            }
        }
    } else if let Some(id_name) = selector.strip_prefix('#') {
        // ID selector
        let search_pattern = format!("id=\"{}\"", id_name);

        if html.contains(&search_pattern) {
            // Extract element with this ID (simplified)
            results.push(format!("Element with id: {}", id_name));
        }
    } else {
        // Tag selector
        let tag = selector;
        let search_pattern = format!("<{}", tag);

        let mut pos = 0;
        while let Some(start) = html[pos..].find(&search_pattern) {
            let absolute_start = pos + start;
            if let Some(end) = html[absolute_start..].find('>') {
                let content_start = absolute_start + end + 1;
                if let Some(content_end) =
                    html[content_start..].find(format!("</{}>", tag).as_str())
                {
                    let content = &html[content_start..content_start + content_end];
                    results.push(content.trim().to_string());
                }
            }
            pos = absolute_start + 1;
        }
    }

    serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())
}

/// 9. html_attr(html, selector, attribute)
/// Extract attribute from elements matching selector
/// html: HTML string
/// selector: CSS selector
/// attribute: attribute name to extract
#[hayashi_fn]
pub fn html_attr(html: String, _selector: String, attribute: String) -> String {
    html_attr_impl(html, attribute)
}

fn html_attr_impl(html: String, attribute: String) -> String {
    let mut attributes = Vec::new();

    // Simplified: search for attribute in all elements
    let search_pattern = format!("{}=\"", attribute);
    let mut pos = 0;

    while let Some(start) = html[pos..].find(&search_pattern) {
        let absolute_start = pos + start + search_pattern.len();
        if let Some(end) = html[absolute_start..].find('"') {
            let attr_value = &html[absolute_start..absolute_start + end];
            attributes.push(attr_value.to_string());
        }
        pos = absolute_start + 1;
    }

    serde_json::to_string(&attributes).unwrap_or_else(|_| "[]".to_string())
}

/// 10. html_text(html, selector)
/// Extract text from elements matching selector
/// html: HTML string
/// selector: CSS selector
#[hayashi_fn]
pub fn html_text(html: String, selector: String) -> String {
    // Simplified: extract text from elements matching selector
    let mut texts = Vec::new();

    if let Some(class_name) = selector.strip_prefix('.') {
        // Class selector - extract text from elements with this class
        let search_pattern = format!("class=\"{}\"", class_name);

        if html.contains(&search_pattern) {
            // Extract text from element (simplified)
            let text = scrape_text_impl(html.clone());
            texts.push(text);
        }
    } else {
        // Tag selector - extract text from all tags
        let tag = selector;
        let search_pattern = format!("<{}", tag);

        let mut pos = 0;
        while let Some(start) = html[pos..].find(&search_pattern) {
            let absolute_start = pos + start;
            if let Some(end) = html[absolute_start..].find('>') {
                let content_start = absolute_start + end + 1;
                if let Some(content_end) =
                    html[content_start..].find(format!("</{}>", tag).as_str())
                {
                    let content = &html[content_start..content_start + content_end];
                    texts.push(content.trim().to_string());
                }
            }
            pos = absolute_start + 1;
        }
    }

    serde_json::to_string(&texts).unwrap_or_else(|_| "[]".to_string())
}

// =============================================================================
// JSON helpers
// =============================================================================

fn serde_to_hayashi(value: &serde_json::Value) -> HayashiValue {
    match value {
        serde_json::Value::Null => HayashiValue::Nil,
        serde_json::Value::Bool(b) => HayashiValue::Bool(*b),
        serde_json::Value::Number(n) => n
            .as_i64()
            .map(HayashiValue::Int)
            .unwrap_or_else(|| HayashiValue::Float(n.as_f64().unwrap_or(f64::NAN))),
        serde_json::Value::String(s) => HayashiValue::Str(s.clone()),
        serde_json::Value::Array(arr) => {
            HayashiValue::List(arr.iter().map(serde_to_hayashi).collect())
        }
        serde_json::Value::Object(obj) => HayashiValue::Dict(
            obj.iter()
                .map(|(k, v)| (k.clone(), serde_to_hayashi(v)))
                .collect(),
        ),
    }
}

fn arrow_to_hayashi_values(array: &arrow::array::ArrayRef) -> Vec<HayashiValue> {
    let len = array.len();
    let mut values = Vec::with_capacity(len);
    match array.data_type() {
        arrow::datatypes::DataType::Float64 => {
            let arr = array
                .as_any()
                .downcast_ref::<arrow::array::Float64Array>()
                .expect("Float64Array");
            for i in 0..len {
                values.push(if arr.is_null(i) {
                    HayashiValue::Nil
                } else {
                    HayashiValue::Float(arr.value(i))
                });
            }
        }
        arrow::datatypes::DataType::Int64 => {
            let arr = array
                .as_any()
                .downcast_ref::<arrow::array::Int64Array>()
                .expect("Int64Array");
            for i in 0..len {
                values.push(if arr.is_null(i) {
                    HayashiValue::Nil
                } else {
                    HayashiValue::Int(arr.value(i))
                });
            }
        }
        arrow::datatypes::DataType::Boolean => {
            let arr = array
                .as_any()
                .downcast_ref::<arrow::array::BooleanArray>()
                .expect("BooleanArray");
            for i in 0..len {
                values.push(if arr.is_null(i) {
                    HayashiValue::Nil
                } else {
                    HayashiValue::Bool(arr.value(i))
                });
            }
        }
        arrow::datatypes::DataType::Utf8 => {
            let arr = array
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .expect("StringArray");
            for i in 0..len {
                values.push(if arr.is_null(i) {
                    HayashiValue::Nil
                } else {
                    HayashiValue::Str(arr.value(i).to_string())
                });
            }
        }
        other => {
            for _ in 0..len {
                values.push(HayashiValue::Nil);
            }
            eprintln!("hayweb: unsupported Arrow type for JSON: {:?}", other);
        }
    }
    values
}

fn hayashi_to_serde(value: &HayashiValue) -> serde_json::Value {
    match value {
        HayashiValue::Nil => serde_json::Value::Null,
        HayashiValue::Float(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(*f).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
        HayashiValue::Int(i) => serde_json::Value::Number((*i).into()),
        HayashiValue::Bool(b) => serde_json::Value::Bool(*b),
        HayashiValue::Str(s) => serde_json::Value::String(s.clone()),
        HayashiValue::List(lst) => {
            serde_json::Value::Array(lst.iter().map(hayashi_to_serde).collect())
        }
        HayashiValue::Dict(map) => serde_json::Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), hayashi_to_serde(v)))
                .collect(),
        ),
        HayashiValue::Arrow(array_ptr, schema_ptr) => {
            let array = unsafe {
                let arr_ptr = *array_ptr as *mut arrow::ffi::FFI_ArrowArray;
                let sch_ptr = *schema_ptr as *mut arrow::ffi::FFI_ArrowSchema;
                match arrow::ffi::from_ffi(std::ptr::read(arr_ptr), &*sch_ptr) {
                    Ok(data) => arrow::array::make_array(data),
                    Err(_) => return serde_json::Value::Null,
                }
            };
            serde_json::Value::Array(
                arrow_to_hayashi_values(&array)
                    .iter()
                    .map(hayashi_to_serde)
                    .collect(),
            )
        }
        _ => serde_json::Value::Null,
    }
}

fn apply_headers(request: reqwest::blocking::RequestBuilder, headers_json: &str) -> reqwest::blocking::RequestBuilder {
    if headers_json.is_empty() {
        return request;
    }
    if let Ok(header_map) = serde_json::from_str::<HashMap<String, String>>(headers_json) {
        let mut r = request;
        for (key, value) in &header_map {
            if let Ok(header_value) = HeaderValue::from_str(value) {
                r = r.header(key, header_value);
            }
        }
        r
    } else {
        request
    }
}

fn http_request_body(
    method: &str,
    url: &str,
    body: Option<String>,
    headers_json: &str,
    timeout: i64,
) -> Result<String, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout.max(1) as u64))
        .build()
        .unwrap_or_else(|_| Client::new());

    let request = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        _ => return Err(format!("unsupported method: {method}")),
    };

    let request = if let Some(b) = body {
        request.body(b)
    } else {
        request
    };

    let request = apply_headers(request, headers_json);

    match request.send() {
        Ok(response) => {
            if response.status().is_success() {
                response.text().map_err(|e| e.to_string())
            } else {
                Err(format!("HTTP {}: {}", response.status().as_u16(), response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

fn http_request_with_retry(
    method: &str,
    url: &str,
    body: Option<String>,
    headers_json: &str,
    timeout: i64,
    max_retries: i64,
    backoff_min: i64,
    backoff_max: i64,
) -> Result<String, String> {
    let mut last_err = String::new();
    let body_opt = body;
    for attempt in 0..max_retries.max(1) {
        match http_request_body(method, url, body_opt.clone(), headers_json, timeout) {
            Ok(text) => return Ok(text),
            Err(e) => {
                last_err = e;
                if attempt >= max_retries.max(1) - 1 {
                    break;
                }
                let min = backoff_min.max(1) as u64;
                let max = (backoff_max as u64).max(min);
                let sleep_secs = if min == max {
                    min
                } else {
                    min + (rand::random::<u64>() % (max - min))
                };
                std::thread::sleep(Duration::from_secs(sleep_secs));
            }
        }
    }
    Err(format!("failed after {max_retries} attempts: {last_err}"))
}

#[hayashi_fn]
pub fn json_parse(json_str: String) -> Result<HayashiValue, String> {
    serde_json::from_str::<serde_json::Value>(&json_str)
        .map(|v| serde_to_hayashi(&v))
        .map_err(|e| e.to_string())
}

#[hayashi_fn]
pub fn json_stringify(value: HayashiValue) -> String {
    hayashi_to_serde(&value).to_string()
}

#[hayashi_fn]
pub fn json_get(value: HayashiValue, path: String) -> HayashiValue {
    let mut current = value;
    if path.trim().is_empty() {
        return current;
    }
    for segment in path.split('/') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        current = match current {
            HayashiValue::Dict(map) => match map.get(segment) {
                Some(v) => v.clone(),
                None => return HayashiValue::Nil,
            },
            HayashiValue::List(lst) => match segment.parse::<usize>() {
                Ok(i) if i < lst.len() => lst[i].clone(),
                _ => return HayashiValue::Nil,
            },
            _ => return HayashiValue::Nil,
        };
    }
    current
}

#[hayashi_fn]
pub fn json_set(
    mut value: HayashiValue,
    path: String,
    new_value: HayashiValue,
) -> Result<HayashiValue, String> {
    let segments: Vec<&str> = path.split('/').map(str::trim).filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Ok(new_value);
    }
    set_nested(&mut value, &segments, new_value)?;
    Ok(value)
}

fn set_nested(
    value: &mut HayashiValue,
    segments: &[&str],
    new_value: HayashiValue,
) -> Result<(), String> {
    if segments.len() == 1 {
        match value {
            HayashiValue::Dict(map) => {
                map.insert(segments[0].to_string(), new_value);
                Ok(())
            }
            HayashiValue::List(lst) => match segments[0].parse::<usize>() {
                Ok(i) if i < lst.len() => {
                    lst[i] = new_value;
                    Ok(())
                }
                _ => Err(format!("index {} out of range", segments[0])),
            },
            _ => Err("json_set target is not dict or list".into()),
        }
    } else {
        match value {
            HayashiValue::Dict(map) => {
                let key = segments[0].to_string();
                let next = map
                    .entry(key)
                    .or_insert_with(|| HayashiValue::Dict(HashMap::new()));
                set_nested(next, &segments[1..], new_value)
            }
            HayashiValue::List(lst) => match segments[0].parse::<usize>() {
                Ok(i) if i < lst.len() => set_nested(&mut lst[i], &segments[1..], new_value),
                _ => Err(format!("index {} out of range", segments[0])),
            },
            _ => Err("json_set target is not dict or list".into()),
        }
    }
}

#[hayashi_fn]
pub fn http_get_body(url: String, headers: String, timeout: i64) -> Result<String, String> {
    http_request_body("GET", &url, None, &headers, timeout)
}

#[hayashi_fn]
pub fn http_post_body(url: String, body: String, headers: String, timeout: i64) -> Result<String, String> {
    http_request_body("POST", &url, Some(body), &headers, timeout)
}

#[hayashi_fn]
pub fn http_get_retry(
    url: String,
    headers: String,
    timeout: i64,
    max_retries: i64,
    backoff_min: i64,
    backoff_max: i64,
) -> Result<String, String> {
    http_request_with_retry("GET", &url, None, &headers, timeout, max_retries, backoff_min, backoff_max)
}

#[hayashi_fn]
pub fn http_post_retry(
    url: String,
    body: String,
    headers: String,
    timeout: i64,
    max_retries: i64,
    backoff_min: i64,
    backoff_max: i64,
) -> Result<String, String> {
    http_request_with_retry(
        "POST",
        &url,
        Some(body),
        &headers,
        timeout,
        max_retries,
        backoff_min,
        backoff_max,
    )
}

#[hayashi_fn]
pub fn sleep(seconds: i64) {
    std::thread::sleep(Duration::from_millis(seconds.max(0) as u64 * 1000));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrape_text_impl() {
        let html = "<html><body><h1>Hello</h1><p>World</p></body></html>";
        let result = scrape_text_impl(html.to_string());
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_scrape_links_impl() {
        let html = "<a href=\"https://example.com\">Link</a><a href=\"/path\">Local</a>";
        let result = scrape_links_impl(html.to_string());
        assert!(result.contains("https://example.com"));
        assert!(result.contains("/path"));
    }

    #[test]
    fn test_scrape_images_impl() {
        let html = "<img src=\"image1.jpg\"><img src=\"/images/image2.png\">";
        let result = scrape_images_impl(html.to_string());
        assert!(result.contains("image1.jpg"));
        assert!(result.contains("/images/image2.png"));
    }

    #[test]
    fn test_html_attr_impl() {
        let html = "<a href=\"https://example.com\" class=\"link\">Link</a>";
        let result = html_attr_impl(html.to_string(), "href".to_string());
        assert!(result.contains("https://example.com"));
    }
}
