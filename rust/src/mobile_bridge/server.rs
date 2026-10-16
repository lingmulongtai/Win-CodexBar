use super::protocol::{Request, encrypt};
use super::{MobileSnapshot, Pairing, is_private_address};
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Semaphore, oneshot};
use tokio::task::JoinHandle;

pub type SnapshotSource = Arc<dyn Fn(&Pairing) -> MobileSnapshot + Send + Sync>;

pub struct MobileServer {
    task: Option<JoinHandle<()>>,
    shutdown: Option<oneshot::Sender<()>>,
    address: SocketAddr,
}

impl MobileServer {
    pub async fn start(
        address: SocketAddr,
        pairing: Pairing,
        source: SnapshotSource,
    ) -> std::io::Result<Self> {
        if !is_private_address(address.ip()) || !pairing.validate() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid mobile bridge configuration",
            ));
        }
        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?;
        let (shutdown, mut stopped) = oneshot::channel();
        let task = tokio::spawn(async move {
            let semaphore = Arc::new(Semaphore::new(16));
            let replay = Arc::new(Mutex::new(VecDeque::<(i64, String)>::new()));
            let mut clients = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    _ = &mut stopped => break,
                    accepted = listener.accept() => {
                        let Ok((socket, peer)) = accepted else { break };
                        if !is_private_address(peer.ip()) { continue; }
                        let Ok(permit) = semaphore.clone().try_acquire_owned() else { continue };
                        let pairing = pairing.clone();
                        let source = source.clone();
                        let replay = replay.clone();
                        clients.spawn(async move {
                            let _permit = permit;
                            let _outcome = tokio::time::timeout(Duration::from_secs(10), handle(socket, pairing, source, replay)).await;
                        });
                    }
                    _ = clients.join_next(), if !clients.is_empty() => {}
                }
            }
            drop(listener);
            clients.abort_all();
            while clients.join_next().await.is_some() {}
        });
        Ok(Self {
            task: Some(task),
            shutdown: Some(shutdown),
            address,
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Return only after the listener and all accepted connections are closed.
    pub async fn stop(mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _sent = shutdown.send(());
        }
        if let Some(task) = self.task.take() {
            let _joined = task.await;
        }
    }
}

impl Drop for MobileServer {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

async fn handle(
    socket: TcpStream,
    pairing: Pairing,
    source: SnapshotSource,
    replay: Arc<Mutex<VecDeque<(i64, String)>>>,
) -> Option<()> {
    let mut socket = BufReader::new(socket);
    let mut line = Vec::with_capacity(1024);
    loop {
        let buffer = socket.fill_buf().await.ok()?;
        if buffer.is_empty() {
            return None;
        }
        let count = buffer
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(buffer.len(), |index| index + 1);
        if line.len() + count > 4096 {
            return None;
        }
        let done = buffer[count - 1] == b'\n';
        line.extend_from_slice(&buffer[..count]);
        socket.consume(count);
        if done {
            break;
        }
    }
    let request: Request = serde_json::from_slice(&line).ok()?;
    let now = chrono::Utc::now().timestamp();
    if !request.authenticate(&pairing, now) {
        return None;
    }
    {
        let mut replay = replay.lock().await;
        replay.retain(|(time, _)| time.abs_diff(now) <= 120);
        if replay.len() >= 4096 || replay.iter().any(|(_, nonce)| nonce == &request.nonce) {
            return None;
        }
        replay.push_back((request.requested_at_epoch_seconds, request.nonce.clone()));
    }
    let plaintext = bounded_snapshot(source(&pairing))?;
    let response = encrypt(
        &pairing,
        &request,
        &plaintext,
        chrono::Utc::now().timestamp(),
    )?;
    let mut response = serde_json::to_vec(&response).ok()?;
    response.push(b'\n');
    socket.get_mut().write_all(&response).await.ok()?;
    socket.get_mut().shutdown().await.ok()?;
    Some(())
}

fn bounded_snapshot(mut snapshot: MobileSnapshot) -> Option<Vec<u8>> {
    // Optional history must never make the existing quota exchange unavailable.
    for _ in 0..9 {
        let raw = serde_json::to_vec(&snapshot).ok()?;
        if raw.len() <= 128 * 1024 {
            return Some(raw);
        }
        for provider in &mut snapshot.providers {
            for series in &mut provider.history {
                if series.samples.len() <= 2 {
                    series.samples.clear();
                    continue;
                }
                let last = series.samples.last()?.t;
                let mut index = 0;
                series.samples.retain(|sample| {
                    let keep = index % 2 == 0 || sample.t == last;
                    index += 1;
                    keep
                });
            }
            provider.history.retain(|series| !series.samples.is_empty());
        }
        if let Some(spend) = &mut snapshot.spend {
            for row in &mut spend.rows {
                if !row.daily.is_empty() {
                    row.daily.drain(..row.daily.len().div_ceil(2));
                }
            }
        }
    }
    snapshot.spend = None;
    let raw = serde_json::to_vec(&snapshot).ok()?;
    (raw.len() <= 128 * 1024).then_some(raw)
}

#[cfg(test)]
mod tests {
    use super::super::protocol::{AUTH_CONTEXT, Envelope, hmac_sha256};
    use super::*;
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

    fn source() -> SnapshotSource {
        Arc::new(|_| MobileSnapshot {
            schema_version: 1,
            source: super::super::SOURCE.into(),
            generated_at_epoch_seconds: chrono::Utc::now().timestamp(),
            providers: vec![],
            spend: None,
        })
    }

    #[test]
    fn optional_history_is_reduced_before_it_can_break_existing_quota_clients() {
        use super::super::{MobileHistoryPoint, MobileProvider, MobileQuotaHistory, MobileWindow};
        let providers = (0..21)
            .map(|index| MobileProvider {
                id: format!("provider{index}"),
                updated_at_epoch_seconds: 1_700_000_000,
                status: "ready".into(),
                subject_id: None,
                windows: vec![MobileWindow {
                    label: "Current".into(),
                    used_fraction: 0.2,
                    resets_at_epoch_seconds: None,
                    window_duration_seconds: None,
                }],
                budget: None,
                balance: None,
                reset_credits: None,
                local_usage: None,
                history_scope_id: Some("A".repeat(43)),
                history: (0..8)
                    .map(|label| MobileQuotaHistory {
                        label: format!("Window {label}"),
                        samples: (0..64)
                            .map(|t| MobileHistoryPoint {
                                t: 1_700_000_000 + t,
                                u: 0.123456789,
                                r: Some(1_700_123_456),
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect();
        let raw = bounded_snapshot(MobileSnapshot {
            schema_version: 1,
            source: super::super::SOURCE.into(),
            generated_at_epoch_seconds: 1_700_000_000,
            providers,
            spend: None,
        })
        .unwrap();
        assert!(raw.len() <= 128 * 1024);
        let decoded: MobileSnapshot = serde_json::from_slice(&raw).unwrap();
        assert_eq!(decoded.providers.len(), 21);
        assert!(
            decoded
                .providers
                .iter()
                .all(|provider| provider.windows.len() == 1)
        );
        assert_eq!(
            decoded.providers[0].history[0].samples.last().unwrap().t,
            1_700_000_063
        );
    }

    #[tokio::test]
    async fn rejects_unauthenticated_and_replayed_requests_and_stops() {
        let pairing = Pairing::generate();
        let server = MobileServer::start("127.0.0.1:0".parse().unwrap(), pairing.clone(), source())
            .await
            .unwrap();
        let mut request = Request {
            protocol_version: 1,
            companion_id: pairing.companion_id.clone(),
            requested_at_epoch_seconds: chrono::Utc::now().timestamp(),
            nonce: URL_SAFE_NO_PAD.encode([1_u8; 16]),
            signature: String::new(),
        };
        let canonical = format!(
            "1\n{}\n{}\n{}\nsnapshot",
            request.companion_id, request.requested_at_epoch_seconds, request.nonce
        );
        let key = hmac_sha256(
            &URL_SAFE_NO_PAD
                .decode(&pairing.shared_key_base64_url)
                .unwrap(),
            AUTH_CONTEXT,
        );
        let address = server.address();
        assert!(exchange(address, &request).await.is_empty());
        request.signature = URL_SAFE_NO_PAD.encode(hmac_sha256(&key, canonical.as_bytes()));
        let response = exchange(address, &request).await;
        let envelope: Envelope = serde_json::from_str(&response).unwrap();
        assert_eq!(envelope.request_nonce, request.nonce);
        assert!(!response.contains("generatedAtEpochSeconds"));
        assert!(exchange(address, &request).await.is_empty());
        let pending = TcpStream::connect(address).await.unwrap();
        server.stop().await;
        assert!(TcpStream::connect(address).await.is_err());
        drop(pending);
        let replacement = MobileServer::start(address, Pairing::generate(), source())
            .await
            .unwrap();
        replacement.stop().await;
    }

    async fn exchange(address: SocketAddr, request: &Request) -> String {
        let mut socket = TcpStream::connect(address).await.unwrap();
        let mut bytes = serde_json::to_vec(request).unwrap();
        bytes.push(b'\n');
        socket.write_all(&bytes).await.unwrap();
        let mut reader = BufReader::new(socket);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        line
    }
}
