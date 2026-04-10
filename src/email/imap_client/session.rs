use anyhow::{Context, Result};
use async_native_tls::{TlsConnector, TlsStream};
use async_std::net::TcpStream;

use crate::config::ImapConfig;

use super::COMMAND_TIMEOUT;

pub(super) type ImapSession = async_imap::Session<TlsStream<TcpStream>>;

pub(super) async fn login(cfg: &ImapConfig) -> Result<ImapSession> {
    let addr = format!("{}:{}", cfg.host, cfg.port);

    let tcp_stream = async_std::future::timeout(COMMAND_TIMEOUT, TcpStream::connect(&addr))
        .await
        .context("Connection timeout")?
        .context("Failed to connect to IMAP server")?;

    let tls = TlsConnector::new();
    let tls_stream =
        async_std::future::timeout(COMMAND_TIMEOUT, tls.connect(&cfg.host, tcp_stream))
            .await
            .context("TLS handshake timeout")?
            .context("TLS connection failed")?;

    let user = cfg.user.as_deref().context("IMAP user is missing")?;
    let client = async_imap::Client::new(tls_stream);
    client
        .login(user, &cfg.password)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to login: {}", e.0))
}
