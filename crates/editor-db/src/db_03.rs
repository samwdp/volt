fn looks_like_sql_server_connection_string(connection_string: &str) -> bool {
    let lower = connection_string.to_ascii_lowercase();
    lower.starts_with("server=")
        || lower.starts_with("data source=")
        || lower.starts_with("jdbc:sqlserver://")
        || lower.starts_with("sqlserver://")
        || lower.contains(";trustservercertificate=")
}

fn looks_like_postgres_connection_string(connection_string: &str) -> bool {
    let lower = connection_string.to_ascii_lowercase();
    lower.starts_with("postgres://")
        || lower.starts_with("postgresql://")
        || lower.contains("host=") && (lower.contains("user=") || lower.contains("dbname="))
}

fn parse_key_value(connection_string: &str, keys: &[&str]) -> Option<String> {
    connection_string
        .split(';')
        .filter_map(|segment| segment.split_once('='))
        .find_map(|(key, value)| {
            keys.iter()
                .any(|expected| key.trim().eq_ignore_ascii_case(expected))
                .then(|| value.trim().to_owned())
        })
}

fn parse_postgres_keyword(connection_string: &str, key: &str) -> Option<String> {
    connection_string
        .split_whitespace()
        .filter_map(|segment| segment.split_once('='))
        .find_map(|(candidate, value)| {
            candidate
                .trim()
                .eq_ignore_ascii_case(key)
                .then(|| value.trim_matches('\'').to_owned())
        })
}

fn parse_url_database(connection_string: &str) -> Option<String> {
    let (_, rest) = connection_string.split_once("://")?;
    let path = rest.split('/').nth(1)?;
    let db = path.split('?').next().unwrap_or(path);
    (!db.is_empty()).then(|| db.to_owned())
}

fn parse_url_host(connection_string: &str) -> Option<String> {
    let (_, rest) = connection_string.split_once("://")?;
    let host_port = rest.split('/').next()?;
    let host_port = host_port.rsplit('@').next().unwrap_or(host_port);
    let host = host_port.split(':').next().unwrap_or(host_port);
    (!host.is_empty()).then(|| host.to_owned())
}

fn sqlite_path(connection_string: &str) -> String {
    let trimmed = connection_string.trim();
    if let Some(path) = trimmed.strip_prefix("sqlite://") {
        return path.to_owned();
    }
    if let Some(path) = trimmed.strip_prefix("sqlite:") {
        return path.trim_start_matches('/').to_owned();
    }
    trimmed.to_owned()
}

fn sqlite_display_label(connection_string: &str) -> String {
    let path = sqlite_path(connection_string);
    Path::new(&path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .unwrap_or(path)
}

fn sqlite_database_name(connection_string: &str) -> Option<String> {
    let path = sqlite_path(connection_string);
    Path::new(&path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_owned)
}

fn quote_sqlite_identifier(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn summarize_sql(sql: &str) -> String {
    let compact = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= 72 {
        compact
    } else {
        let mut shortened = compact.chars().take(69).collect::<String>();
        shortened.push_str("...");
        shortened
    }
}

fn sanitize_file_component(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned();
    if sanitized.is_empty() {
        "db".to_owned()
    } else {
        sanitized
    }
}

fn escape_bracket(value: &str) -> String {
    value.replace(']', "]]")
}

fn escape_double_quote(value: &str) -> String {
    value.replace('"', "\"\"")
}

#[cfg(test)]
mod tests;
