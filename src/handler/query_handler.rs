use mysql::*;
use mysql::prelude::*;
use serde_json::Value;

fn get_conn() -> PooledConn {
    let url = "mysql://root:your_password@localhost:3306/search_engine";
    let pool = Pool::new(url).expect("DB pool failed");
    pool.get_conn().expect("Failed to get conn")
}

fn tokenize(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|w| !w.is_empty())
        .collect()
}

pub fn handle_query(query: &str) -> Vec<String> {
    let words = tokenize(query);
    let mut conn = get_conn();
    let mut urls = Vec::new();

    for word in words {
        let json_urls_opt: Option<String> = conn.exec_first(
            "SELECT urls FROM word_index WHERE word = :word",
            params! { "word" => &word },
        ).unwrap_or(None);

        if let Some(json_urls) = json_urls_opt {
            // Parse JSON array string to Vec<String>
            if let Ok(parsed) = serde_json::from_str::<Vec<String>>(&json_urls) {
                urls.extend(parsed);
            }
        }
    }

    urls
}

