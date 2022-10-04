use crate::common::TlsServerConfigError;
use rustls::ServerConfig as TlsServerConfig;
use std::io::BufReader;
use tokio::fs;
use tokio_rustls::rustls::{self, Certificate, PrivateKey};

///加载服务端tls配置
pub async fn load_tls_config(
    cert_path: &str,
    key_path: &str,
) -> Result<TlsServerConfig, TlsServerConfigError> {
    let certs = load_certs(cert_path).await?;
    let mut keys = load_keys(key_path).await?;
    let tls_config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.remove(0))?;
    Ok(tls_config)
}

async fn load_certs(path: &str) -> Result<Vec<Certificate>, TlsServerConfigError> {
    let cert_data = fs::read(path)
        .await
        .map_err(TlsServerConfigError::LoadCert)?;
    let mut cert_reader = BufReader::new(cert_data.as_slice());
    let mut certs =
        rustls_pemfile::certs(&mut cert_reader).map_err(TlsServerConfigError::ParseCert)?;
    let certs = certs.drain(..).map(Certificate).collect();
    Ok(certs)
}

async fn load_keys(path: &str) -> Result<Vec<PrivateKey>, TlsServerConfigError> {
    let key_data = fs::read(path)
        .await
        .map_err(TlsServerConfigError::LoadKey)?;
    let mut key_reader = BufReader::new(key_data.as_slice());
    let mut keys = rustls_pemfile::rsa_private_keys(&mut key_reader)
        .map_err(TlsServerConfigError::ParseKey)?;
    let keys = keys.drain(..).map(PrivateKey).collect();
    Ok(keys)
}
