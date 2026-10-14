//! Explicit import only. Refresh never opens another application's session store.
use crate::browser::cookies::CookieExtractor;
use std::{collections::BTreeMap, path::Path};

const UNAVAILABLE: &str =
    "Close the Google Gemini desktop app, then connect again. Its session database is unavailable.";

pub fn import() -> Result<String, String> {
    let root = dirs::data_dir().ok_or(UNAVAILABLE)?.join("Gemini");
    import_from(&root)
}

fn import_from(root: &Path) -> Result<String, String> {
    let db = root.join("Network").join("Cookies");
    if !db.is_file() {
        return Err("Install and sign in to the official Google Gemini desktop app first.".into());
    }
    let connection =
        crate::core::open_readonly_sqlite_connection(&db, crate::core::DEFAULT_SQLITE_BUSY_TIMEOUT)
            .map_err(|_| UNAVAILABLE.to_string())?;
    let key = CookieExtractor::get_chromium_encryption_key(&root.join("Local State")).ok();
    read(&connection, key.as_deref().unwrap_or(&[]))
}

fn read(connection: &rusqlite::Connection, key: &[u8]) -> Result<String, String> {
    let mut query = connection
        .prepare(
            "SELECT name, value, encrypted_value FROM cookies WHERE host_key IN
        ('.google.com', 'gemini.google.com', '.gemini.google.com')
        AND path = '/' AND (expires_utc = 0 OR expires_utc > ?1)
        ORDER BY length(host_key) DESC, length(path) DESC",
        )
        .map_err(|_| UNAVAILABLE.to_string())?;
    let chrome_now = (chrono::Utc::now().timestamp() + 11_644_473_600) * 1_000_000;
    let rows = query
        .query_map([chrome_now], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })
        .map_err(|_| UNAVAILABLE.to_string())?;
    let mut cookies = BTreeMap::new();
    for row in rows {
        let (name, plain, encrypted) = row.map_err(|_| UNAVAILABLE.to_string())?;
        if !valid_name(&name) {
            return Err("Gemini session contains an invalid cookie name.".into());
        }
        let value = if !plain.is_empty() {
            plain
        } else {
            CookieExtractor::decrypt_chromium_cookie(&encrypted, key)
                .map_err(|_| "Gemini session encryption is unsupported. App-bound encryption cannot be imported.".to_string())?
        };
        if value.is_empty() {
            continue;
        }
        if value.bytes().any(|b| b < 0x20 || b == 0x7f || b == b';') {
            return Err("Gemini session contains an invalid cookie value.".into());
        }
        if let Some(old) = cookies.insert(name, value.clone())
            && old != value
        {
            return Err("Conflicting Gemini session scopes. Reconnect after signing in with one primary Google account.".into());
        }
    }
    if cookies.is_empty() {
        return Err("Sign in to the Google Gemini desktop app before connecting.".into());
    }
    Ok(cookies
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; "))
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn db() -> rusqlite::Connection {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE cookies (host_key TEXT, path TEXT, name TEXT, value TEXT, encrypted_value BLOB, expires_utc INTEGER);").unwrap();
        db
    }
    fn add(
        db: &rusqlite::Connection,
        host: &str,
        path: &str,
        name: &str,
        value: &str,
        expiry: i64,
    ) {
        db.execute(
            "INSERT INTO cookies VALUES (?1,?2,?3,?4,x'',?5)",
            rusqlite::params![host, path, name, value, expiry],
        )
        .unwrap();
    }
    #[test]
    fn reads_only_gemini_scopes_and_unexpired_values() {
        let db = db();
        add(&db, ".google.com", "/", "SID", "test-session", 0);
        add(&db, "gemini.google.com", "/", "app", "test-app", 0);
        add(&db, "gemini.google.com", "/app", "page-only", "excluded", 0);
        add(&db, "google.com", "/", "parent-host-only", "excluded", 0);
        add(&db, "mail.google.com", "/", "gmail", "private", 0);
        add(&db, ".google.com", "/mail", "mail", "private", 0);
        add(&db, ".google.com", "/", "expired", "old", 1);
        assert_eq!(read(&db, &[]).unwrap(), "SID=test-session; app=test-app");
    }
    #[test]
    fn refuses_conflicting_scope_and_header_injection() {
        let db = db();
        add(&db, ".google.com", "/", "SID", "one", 0);
        add(&db, "gemini.google.com", "/", "SID", "two", 0);
        assert!(read(&db, &[]).is_err());
        let db = super::tests::db();
        add(&db, ".google.com", "/", "SID", "value\r\nInjected: true", 0);
        assert!(read(&db, &[]).is_err());
    }
    #[test]
    fn missing_session_does_not_read_other_providers() {
        assert!(read(&db(), &[]).is_err());
    }
}
