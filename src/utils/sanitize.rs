/// Placeholder for sensitive info filtering
pub fn filter_sensitive(content: &str) -> String {
    let mut result = content.to_string();

    // API keys
    let patterns = [
        (r"sk-[a-zA-Z0-9]{20,}", "***API_KEY***"),
        (r"ghp_[a-zA-Z0-9]{36}", "***GITHUB_TOKEN***"),
    ];

    for (pattern, replacement) in &patterns {
        if let Ok(re) = regex_lite::Regex::new(pattern) {
            result = re.replace_all(&result, *replacement).to_string();
        }
    }

    result
}
