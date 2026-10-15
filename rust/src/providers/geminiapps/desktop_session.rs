//! Explicit import only. Refresh never opens another application's session store.
use crate::browser::cookies::CookieExtractor;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use sha2::{Digest, Sha256};
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
    let version: i64 = connection
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM meta WHERE key = 'version'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "Gemini session database version is unavailable.".to_string())?;
    if !(1..=24).contains(&version) {
        return Err("Gemini session database version is unsupported.".into());
    }
    let mut query = connection
        .prepare(
            "SELECT name, value, encrypted_value, host_key FROM cookies WHERE host_key IN
        ('.google.com', 'gemini.google.com', '.gemini.google.com')
        AND path = '/' AND (expires_utc = 0 OR expires_utc > ?1)
        ORDER BY length(host_key) DESC, length(path) DESC",
        )
        .map_err(|_| UNAVAILABLE.to_string())?;
    let chrome_now = chrono::Utc::now().timestamp_micros() + 11_644_473_600_000_000;
    let rows = query
        .query_map([chrome_now], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|_| UNAVAILABLE.to_string())?;
    let mut cookies = BTreeMap::new();
    for row in rows {
        let (name, plain, encrypted, host) = row.map_err(|_| UNAVAILABLE.to_string())?;
        if !valid_name(&name) {
            return Err("Gemini session contains an invalid cookie name.".into());
        }
        let value = if !plain.is_empty() {
            plain
        } else {
            decrypt(&encrypted, key, &host, version)?
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

fn decrypt(encrypted: &[u8], key: &[u8], host: &str, version: i64) -> Result<String, String> {
    let unavailable = || {
        "Gemini session encryption is unsupported or invalid. App-bound encryption cannot be imported.".to_string()
    };
    if encrypted.starts_with(b"v20") {
        return Err(unavailable());
    }
    let plaintext = if encrypted.starts_with(b"v10") || encrypted.starts_with(b"v11") {
        if encrypted.len() < 31 {
            return Err(unavailable());
        }
        Aes256Gcm::new_from_slice(key)
            .map_err(|_| unavailable())?
            .decrypt(Nonce::from_slice(&encrypted[3..15]), &encrypted[15..])
            .map_err(|_| unavailable())?
    } else {
        CookieExtractor::dpapi_decrypt(encrypted).map_err(|_| unavailable())?
    };
    // Chromium database v24 binds encrypted cookie bytes to their exact host.
    // Do not guess a metadata prefix from printable/non-printable characters.
    let value = if version >= 24 {
        let digest = Sha256::digest(host.as_bytes());
        if !plaintext.starts_with(&digest) {
            return Err("Gemini session domain verification failed.".into());
        }
        &plaintext[32..]
    } else {
        &plaintext[..]
    };
    String::from_utf8(value.to_vec()).map_err(|_| unavailable())
}

pub(super) fn valid_name(name: &str) -> bool {
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
        db.execute_batch("CREATE TABLE cookies (host_key TEXT, path TEXT, name TEXT, value TEXT, encrypted_value BLOB, expires_utc INTEGER); CREATE TABLE meta (key TEXT, value TEXT); INSERT INTO meta VALUES ('version', '24');").unwrap();
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

    fn encrypted(plaintext: &[u8], prefix: &[u8]) -> Vec<u8> {
        let nonce = [7; 12];
        let cipher = Aes256Gcm::new_from_slice(&[42; 32]).unwrap();
        let mut blob = prefix.to_vec();
        blob.extend_from_slice(&nonce);
        blob.extend(
            cipher
                .encrypt(Nonce::from_slice(&nonce), plaintext)
                .unwrap(),
        );
        blob
    }

    #[test]
    fn validates_encrypted_host_binding_and_rejects_truncated_digest() {
        let host = ".google.com";
        let mut value = Sha256::digest(host.as_bytes()).to_vec();
        value.extend_from_slice(b"test-session");
        for prefix in [b"v10", b"v11"] {
            let blob = encrypted(&value, prefix);
            assert_eq!(decrypt(&blob, &[42; 32], host, 24).unwrap(), "test-session");
            assert!(decrypt(&blob, &[42; 32], "gemini.google.com", 24).is_err());
            assert!(decrypt(&blob, &[41; 32], host, 24).is_err());
        }
        assert!(decrypt(&encrypted(&value[..20], b"v10"), &[42; 32], host, 24).is_err());
        assert!(decrypt(b"v10short", &[42; 32], host, 24).is_err());
        assert!(decrypt(b"v20unsupported", &[42; 32], host, 24).is_err());
        assert_eq!(
            decrypt(&encrypted(b"legacy", b"v10"), &[42; 32], host, 23).unwrap(),
            "legacy"
        );
    }

    #[test]
    fn checks_row_host_and_refuses_unknown_database_versions() {
        let db = db();
        let mut value = Sha256::digest(b".google.com").to_vec();
        value.extend_from_slice(b"test-session");
        db.execute(
            "INSERT INTO cookies VALUES ('gemini.google.com', '/', 'SID', '', ?1, 0)",
            [encrypted(&value, b"v10")],
        )
        .unwrap();
        assert!(read(&db, &[42; 32]).is_err());
        db.execute("UPDATE cookies SET host_key = '.google.com'", [])
            .unwrap();
        assert_eq!(read(&db, &[42; 32]).unwrap(), "SID=test-session");
        db.execute("UPDATE meta SET value = '25'", []).unwrap();
        assert!(read(&db, &[42; 32]).is_err());
    }
}
