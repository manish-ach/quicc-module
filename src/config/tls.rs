use anyhow::Result;
use quinn::{ClientConfig, ServerConfig};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use sha2::{Digest, Sha256};

pub struct CertInfo {
    pub server_config: ServerConfig,
    pub fingerprint: String,
}

pub fn build_server_config() -> Result<CertInfo> {
    let certified_key = generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_der = certified_key.cert.der().to_vec();
    let priv_key = certified_key.signing_key.serialize_der();

    let cert_chain = vec![CertificateDer::from(cert_der.clone())];
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(priv_key));

    let server_config = ServerConfig::with_single_cert(cert_chain, key)?;

    let mut hasher = Sha256::new();
    hasher.update(&cert_der);
    let fingerprint = hex::encode(hasher.finalize());

    Ok(CertInfo {
        server_config,
        fingerprint,
    })
}

pub fn insecure_client_config() -> ClientConfig {
    use rustls::{
        SignatureScheme,
        client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    };
    use std::sync::Arc;

    #[derive(Debug)]
    struct AllowAnyCert;

    impl ServerCertVerifier for AllowAnyCert {
        fn verify_server_cert(
            &self,
            end_entity: &CertificateDer<'_>,
            intermediates: &[CertificateDer<'_>],
            server_name: &rustls::pki_types::ServerName<'_>,
            ocsp_response: &[u8],
            now: rustls::pki_types::UnixTime,
        ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error>
        {
            Ok(ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            message: &[u8],
            cert: &CertificateDer<'_>,
            dss: &rustls::DigitallySignedStruct,
        ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
        {
            Ok(HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            message: &[u8],
            cert: &CertificateDer<'_>,
            dss: &rustls::DigitallySignedStruct,
        ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
        {
            Ok(HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            vec![
                SignatureScheme::RSA_PKCS1_SHA256,
                SignatureScheme::ECDSA_NISTP256_SHA256,
                SignatureScheme::ED25519,
            ]
        }
    }

    // Build rustls ClientConfig with dangerous verifier
    let rustls_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AllowAnyCert))
        .with_no_client_auth();

    // Wrap rustls config in Quinn's QuicClientConfig
    let quic_crypto = quinn::crypto::rustls::QuicClientConfig::try_from(rustls_config)
        .expect("Failed to create QuicClientConfig");

    // Create Quinn ClientConfig
    ClientConfig::new(Arc::new(quic_crypto))
}
