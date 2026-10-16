//! Authenticated settings operations, separate from the immutable v1 snapshot protocol.
use super::protocol::{Envelope, Pairing, constant_time_eq, hmac_sha256};
use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng, Payload, rand_core::RngCore},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ControlRequest {
    pub protocol_version: u8,
    pub companion_id: String,
    pub requested_at_epoch_seconds: i64,
    pub nonce: String,
    pub resource: String,
    pub payload: String,
    pub signature: String,
}

impl ControlRequest {
    fn payload_bytes(&self) -> Option<Vec<u8>> {
        let raw = URL_SAFE_NO_PAD.decode(&self.payload).ok()?;
        (raw.len() <= 2048).then_some(raw)
    }
    fn payload_hash(&self) -> Option<String> {
        Some(URL_SAFE_NO_PAD.encode(Sha256::digest(self.payload_bytes()?)))
    }
    fn canonical(&self) -> Option<String> {
        Some(format!(
            "2\n{}\n{}\n{}\n{}\n{}",
            self.companion_id,
            self.requested_at_epoch_seconds,
            self.nonce,
            self.resource,
            self.payload_hash()?
        ))
    }
    pub(super) fn authenticate(&self, pairing: &Pairing, now: i64) -> Option<Vec<u8>> {
        if self.protocol_version != 2
            || self.companion_id != pairing.companion_id
            || self.requested_at_epoch_seconds.abs_diff(now) > 120
            || !matches!(self.resource.as_str(), "settings.get" | "settings.update")
            || URL_SAFE_NO_PAD.decode(&self.nonce).ok()?.len() != 16
        {
            return None;
        }
        let key = hmac_sha256(&pairing.master_key()?, b"codexbar-windows-auth-v2");
        let expected = hmac_sha256(&key, self.canonical()?.as_bytes());
        if !constant_time_eq(&URL_SAFE_NO_PAD.decode(&self.signature).ok()?, &expected) {
            return None;
        }
        self.payload_bytes()
    }
    pub(super) fn encrypt(
        &self,
        pairing: &Pairing,
        plaintext: &[u8],
        now: i64,
    ) -> Option<Envelope> {
        let key = hmac_sha256(&pairing.master_key()?, b"codexbar-windows-encryption-v2");
        let cipher = Aes256Gcm::new_from_slice(&key).ok()?;
        let mut iv = [0u8; 12];
        OsRng.fill_bytes(&mut iv);
        let aad = format!(
            "2\n{}\n{}\n{now}\n{}\n{}",
            self.companion_id,
            self.nonce,
            self.resource,
            self.payload_hash()?
        );
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&iv),
                Payload {
                    msg: plaintext,
                    aad: aad.as_bytes(),
                },
            )
            .ok()?;
        Some(Envelope {
            protocol_version: 2,
            companion_id: self.companion_id.clone(),
            request_nonce: self.nonce.clone(),
            sent_at_epoch_seconds: now,
            iv: URL_SAFE_NO_PAD.encode(iv),
            ciphertext: URL_SAFE_NO_PAD.encode(ciphertext),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn controls_bind_exact_payload_resource_and_version() {
        let pairing = Pairing::generate();
        let mut request = ControlRequest {
            protocol_version: 2,
            companion_id: pairing.companion_id.clone(),
            requested_at_epoch_seconds: 1700000000,
            nonce: URL_SAFE_NO_PAD.encode([1u8; 16]),
            resource: "settings.update".into(),
            payload: URL_SAFE_NO_PAD.encode(br#"{"refreshIntervalSecs":60}"#),
            signature: String::new(),
        };
        let key = hmac_sha256(&pairing.master_key().unwrap(), b"codexbar-windows-auth-v2");
        request.signature =
            URL_SAFE_NO_PAD.encode(hmac_sha256(&key, request.canonical().unwrap().as_bytes()));
        assert!(request.authenticate(&pairing, 1700000000).is_some());
        let saved = request.payload.clone();
        request.payload = URL_SAFE_NO_PAD.encode(br#"{"refreshIntervalSecs":0}"#);
        assert!(request.authenticate(&pairing, 1700000000).is_none());
        request.payload = saved;
        request.resource = "settings.get".into();
        assert!(request.authenticate(&pairing, 1700000000).is_none());
        request.resource = "settings.update".into();
        request.protocol_version = 1;
        assert!(request.authenticate(&pairing, 1700000000).is_none());
    }
}
