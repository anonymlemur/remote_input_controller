// TLS and certificate utilities module
use std::{fs::File, io::Error as IoError};
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};

/// Load certificates from a PEM file
pub fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>, IoError> {
    let mut reader = std::io::BufReader::new(File::open(path)?);
    rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
}

/// Load private key from a PEM file
pub fn load_private_key(path: &str) -> Result<PrivateKeyDer<'static>, IoError> {
    let mut reader = std::io::BufReader::new(File::open(path)?);
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .collect::<Result<Vec<_>, _>>()?;
    let pkcs8 = keys.into_iter().next().ok_or_else(|| {
        IoError::new(std::io::ErrorKind::InvalidInput, "No private key found")
    })?;
    Ok(PrivateKeyDer::from(pkcs8))
}
