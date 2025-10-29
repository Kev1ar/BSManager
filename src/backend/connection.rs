use anyhow::Context;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_tungstenite::{client_async, WebSocketStream};
use tokio_rustls::rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName};
use webpki_roots::TLS_SERVER_ROOTS;
use url::Url;

pub async fn connect_wss(
    url: &str,
) -> anyhow::Result<WebSocketStream<TlsStream<TcpStream>>> {
    // parse url and extract host/port
    let parsed = Url::parse(url).context("parsing url")?;
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("missing host"))?;
    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| anyhow::anyhow!("missing port"))?;

    if parsed.scheme() != "wss" {
        return Err(anyhow::anyhow!("only wss:// URLs are supported by this function"));
    }

    // Build RootCertStore from webpki roots
    let mut roots = RootCertStore::empty();
    roots.add_server_trust_anchors(
        TLS_SERVER_ROOTS
            .0
            .iter()
            .map(|ta| OwnedTrustAnchor::from_subject_spki_name_constraints(ta.subject, ta.spki, ta.name_constraints)),
    );

    // Build rustls ClientConfig
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let server_name = ServerName::try_from(host).context("invalid server name")?;

    // TCP connect
    let tcp = TcpStream::connect((host, port)).await.context("tcp connect failed")?;

    // TLS handshake
    let tls_stream = connector.connect(server_name, tcp).await.context("tls handshake failed")?;

    // WebSocket client handshake over TLS stream
    let (ws_stream, _resp) = client_async(parsed, tls_stream).await.context("websocket handshake failed")?;

    Ok(ws_stream)
}
