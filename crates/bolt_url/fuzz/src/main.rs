fn main() {
    afl::fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            if let Ok(path) = bolt_url::UrlPath::parse(&s) {
                let _ = path.sanitized_path();
            }
        }
    });
}