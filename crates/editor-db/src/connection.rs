pub(crate) fn parse_connect_prompt(input: &str) -> Result<(Option<String>, String), String> {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix("remember ") {
        let Some((alias, connection_string)) = rest.split_once("::") else {
            return Err(
                "remembered connections must use `remember <alias> :: <connection string>`"
                    .to_owned(),
            );
        };
        let alias = alias.trim();
        if alias.is_empty() {
            return Err("remembered connection alias is empty".to_owned());
        }
        let connection_string = connection_string.trim();
        if connection_string.is_empty() {
            return Err("remembered connection string is empty".to_owned());
        }
        return Ok((Some(alias.to_owned()), connection_string.to_owned()));
    }
    Ok((None, trimmed.to_owned()))
}

/// Parses one DB connect prompt payload.
pub fn parse_db_connect_prompt(input: &str) -> Result<(Option<String>, String), String> {
    parse_connect_prompt(input)
}

pub(crate) fn looks_like_sql_server_connection_string(connection_string: &str) -> bool {
    let lower = connection_string.to_ascii_lowercase();
    lower.starts_with("server=")
        || lower.starts_with("data source=")
        || lower.starts_with("jdbc:sqlserver://")
        || lower.starts_with("sqlserver://")
        || lower.contains(";trustservercertificate=")
}

pub(crate) fn looks_like_postgres_connection_string(connection_string: &str) -> bool {
    let lower = connection_string.to_ascii_lowercase();
    lower.starts_with("postgres://")
        || lower.starts_with("postgresql://")
        || lower.contains("host=") && (lower.contains("user=") || lower.contains("dbname="))
}

pub(crate) fn parse_key_value(connection_string: &str, keys: &[&str]) -> Option<String> {
    connection_string
        .split(';')
        .filter_map(|segment| segment.split_once('='))
        .find_map(|(key, value)| {
            keys.iter()
                .any(|expected| key.trim().eq_ignore_ascii_case(expected))
                .then(|| value.trim().to_owned())
        })
}

pub(crate) fn parse_postgres_keyword(connection_string: &str, key: &str) -> Option<String> {
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

pub(crate) fn parse_url_database(connection_string: &str) -> Option<String> {
    let (_, rest) = connection_string.split_once("://")?;
    let path = rest.split('/').nth(1)?;
    let db = path.split('?').next().unwrap_or(path);
    (!db.is_empty()).then(|| db.to_owned())
}

pub(crate) fn parse_url_host(connection_string: &str) -> Option<String> {
    let (_, rest) = connection_string.split_once("://")?;
    let host_port = rest.split('/').next()?;
    let host_port = host_port.rsplit('@').next().unwrap_or(host_port);
    let host = host_port.split(':').next().unwrap_or(host_port);
    (!host.is_empty()).then(|| host.to_owned())
}

pub(crate) fn escape_double_quote(value: &str) -> String {
    value.replace('"', "\"\"")
}
