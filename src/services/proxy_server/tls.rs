use crate::common::ServerConfig;
use rustls::Error as TlsError;
use std::io::{BufReader, Error as IoError};
use std::sync::Arc;
use thiserror::Error;
use tokio::fs;
use tokio_rustls::{
    rustls::{self, Certificate, PrivateKey},
    TlsAcceptor,
};

#[derive(Error, Debug)]
pub enum LoadAcceptorError {
    #[error("load public cert failed, {0}")]
    LoadCert(IoError),
    #[error("parse public cert failed, {0}")]
    ParseCert(IoError),
    #[error("load private key failed, {0}")]
    LoadKey(IoError),
    #[error("parse private key failed, {0}")]
    ParseKey(IoError),
    #[error("tls error, {0}")]
    TlsErr(#[from] TlsError),
}

pub async fn load_tls_acceptor(
    server_config: &ServerConfig,
) -> Result<TlsAcceptor, LoadAcceptorError> {
    let certs = load_certs(&server_config.public_key_path).await?;
    let mut keys = load_keys(&server_config.private_key_path).await?;
    let tls_config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.remove(0))?;
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));
    Ok(acceptor)
}

async fn load_certs(path: &str) -> Result<Vec<Certificate>, LoadAcceptorError> {
    let cert_data = fs::read(path).await.map_err(LoadAcceptorError::LoadCert)?;
    let mut cert_reader = BufReader::new(cert_data.as_slice());
    let mut certs =
        rustls_pemfile::certs(&mut cert_reader).map_err(LoadAcceptorError::ParseCert)?;
    let certs = certs.drain(..).map(Certificate).collect();
    Ok(certs)
}

async fn load_keys(path: &str) -> Result<Vec<PrivateKey>, LoadAcceptorError> {
    let key_data = fs::read(path).await.map_err(LoadAcceptorError::LoadKey)?;
    let mut key_reader = BufReader::new(key_data.as_slice());
    let mut keys =
        rustls_pemfile::rsa_private_keys(&mut key_reader).map_err(LoadAcceptorError::ParseKey)?;
    let keys = keys.drain(..).map(PrivateKey).collect();
    Ok(keys)
}
