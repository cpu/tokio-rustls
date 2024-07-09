mod utils {
    use std::io::{BufReader, Cursor, IoSlice};
    use std::sync::Arc;

    use rustls::{ClientConfig, RootCertStore, ServerConfig};
    use rustls_pemfile::{certs, private_key};
    use tokio::io::{self, AsyncWrite, AsyncWriteExt};

    #[allow(dead_code)]
    pub fn make_configs() -> (Arc<ServerConfig>, Arc<ClientConfig>) {
        let cert = certs(&mut BufReader::new(Cursor::new(CHAIN)))
            .map(|result| result.unwrap())
            .collect();
        let key = private_key(&mut BufReader::new(Cursor::new(EE_KEY)))
            .unwrap().unwrap();
        let sconfig = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert, key.into())
            .unwrap();

        let mut client_root_cert_store = RootCertStore::empty();
        let mut root = BufReader::new(Cursor::new(ROOT));
        for cert in certs(&mut root) {
            client_root_cert_store.add(cert.unwrap()).unwrap();
        }

        let cconfig = ClientConfig::builder()
            .with_root_certificates(client_root_cert_store)
            .with_no_client_auth();

        (Arc::new(sconfig), Arc::new(cconfig))
    }

    #[allow(dead_code)]
    pub async fn write<W: AsyncWrite + Unpin>(
        w: &mut W,
        data: &[u8],
        vectored: bool,
    ) -> io::Result<()> {
        if !vectored {
            return w.write_all(data).await;
        }

        let mut data = data;

        while !data.is_empty() {
            let chunk_size = (data.len() / 4).max(1);
            let vectors = data
                .chunks(chunk_size)
                .map(IoSlice::new)
                .collect::<Vec<_>>();
            let written = w.write_vectored(&vectors).await?;
            data = &data[written..];
        }

        Ok(())
    }

    /// A test root certificate that is the trust anchor for the [`CHAIN`].
    pub const ROOT: &str = include_str!("certs/root.pem");
    /// A server certificate chain that includes both an end-entity server certificate
    /// and the intermediate certificate that issued it. The [`ROOT`] is configured
    /// out-of-band.
    pub const CHAIN: &str = include_str!("certs/chain.pem");
    /// A private key corresponding to the end-entity server certificate in [`CHAIN`].
    pub const EE_KEY: &str = include_str!("certs/end.key");
}
