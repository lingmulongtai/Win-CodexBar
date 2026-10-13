//! Endpoint discovery only; never changes Tailscale routing or an exit node.
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::time::Duration;

use codexbar::host::{CommandOptions, CommandResult, CommandRunner};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MobileNetworkAddresses {
    tailscale_address: Option<String>,
    lan_address: Option<String>,
}

fn tailscale_address(result: &CommandResult) -> Option<Ipv4Addr> {
    if result.exit_code != Some(0) || result.timed_out {
        return None;
    }
    let address: Ipv4Addr = result.text.trim().parse().ok()?;
    is_tailscale(address).then_some(address)
}

fn is_tailscale(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    octets[0] == 100 && (64..=127).contains(&octets[1])
}

#[tauri::command]
pub(crate) async fn get_mobile_network_addresses() -> MobileNetworkAddresses {
    let options = CommandOptions {
        timeout: Duration::from_secs(5),
        initial_delay: Duration::ZERO,
        settle_after_stop: Duration::ZERO,
        extra_args: vec!["ip".into(), "-4".into()],
        ..CommandOptions::default()
    };
    let tailscale_address = CommandRunner::new()
        .run_async("tailscale", None, &options)
        .await
        .ok()
        .and_then(|result| tailscale_address(&result))
        .map(|address| address.to_string());
    let lan_address = UdpSocket::bind("0.0.0.0:0")
        .ok()
        .and_then(|socket| {
            socket.connect("192.0.2.1:9").ok()?;
            Some(socket.local_addr().ok()?.ip())
        })
        .and_then(|address| match address {
            IpAddr::V4(ip) if ip.is_private() && !is_tailscale(ip) => Some(ip.to_string()),
            _ => None,
        });
    MobileNetworkAddresses {
        tailscale_address,
        lan_address,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(text: &str) -> CommandResult {
        CommandResult {
            text: text.into(),
            stderr: String::new(),
            timed_out: false,
            exit_code: Some(0),
        }
    }

    #[test]
    fn discovers_self_address_without_exit_node_or_default_route_information() {
        assert_eq!(
            tailscale_address(&result("100.74.102.71\r\n")),
            Some(Ipv4Addr::new(100, 74, 102, 71))
        );
        for text in [
            "",
            "192.168.1.10",
            "8.8.8.8",
            "100.128.0.1",
            "fd7a:115c:a1e0::1",
            "100.74.102.71\n100.80.1.2",
        ] {
            assert!(tailscale_address(&result(text)).is_none(), "{text}");
        }
    }

    #[test]
    fn rejects_failed_or_timed_out_cli_results() {
        let mut output = result("100.74.102.71");
        output.exit_code = Some(1);
        assert!(tailscale_address(&output).is_none());
        output.exit_code = Some(0);
        output.timed_out = true;
        assert!(tailscale_address(&output).is_none());
    }
}
