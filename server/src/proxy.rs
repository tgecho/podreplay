use std::net::{IpAddr, SocketAddr};

use axum::{
    body::{Body, BoxBody},
    extract::{ConnectInfo, Extension},
    handler::Handler,
    response::IntoResponse,
};
use headers::{HeaderMap, HeaderName, HeaderValue};
use hyper::{
    client::{HttpConnector, ResponseFuture},
    header::{self, Entry, InvalidHeaderValue, ToStrError},
    Client, Request, Response, StatusCode, Uri,
};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct ProxyClient {
    client: Client<HttpsConnector<HttpConnector>>,
}

impl ProxyClient {
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        Self { client }
    }

    fn request(&self, req: Request<Body>) -> ResponseFuture {
        self.client.request(req)
    }
}

impl Default for ProxyClient {
    fn default() -> Self {
        Self::new()
    }
}

pub fn proxy_to(
    target_uri: Uri,
) -> impl Handler<(
    Extension<ProxyClient>,
    ConnectInfo<SocketAddr>,
    Request<Body>,
)> {
    let target_host = target_uri
        .host()
        .and_then(|h| h.parse::<HeaderValue>().ok());
    |Extension(client), ConnectInfo(addr): ConnectInfo<SocketAddr>, request| async move {
        proxy_request(target_host, target_uri, addr.ip(), request, client).await
    }
}

#[tracing::instrument]
async fn proxy_request(
    target_host: Option<HeaderValue>,
    target_uri: Uri,
    client_ip: IpAddr,
    mut request: Request<Body>,
    client: ProxyClient,
) -> Result<Response<Body>, ProxyError> {
    *request.uri_mut() = target_uri;

    let headers = request.headers_mut();
    remove_hop_headers(headers);
    if let Some(host) = target_host {
        headers.insert(header::HOST, host);
    }
    match headers.entry("forwarded") {
        Entry::Vacant(entry) => {
            entry.insert(client_ip.to_string().parse()?);
        }
        Entry::Occupied(mut entry) => {
            let addr = format!("{}, {}", entry.get().to_str()?, client_ip);
            entry.insert(addr.parse()?);
        }
    }

    tracing::debug!("proxy: {:?}", request);

    let mut response = client.request(request).await?;
    let response_headers = response.headers_mut();
    remove_hop_headers(response_headers);
    Ok(response)
}

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("{0}")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[error("{0}")]
    ToStrError(#[from] ToStrError),
    #[error("{0}")]
    FetchError(#[from] hyper::Error),
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response<BoxBody> {
        tracing::error!(?self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

/// Returns a clone of the headers without the [hop-by-hop headers].
///
/// [hop-by-hop headers]: http://www.w3.org/Protocols/rfc2616/rfc2616-sec13.html
fn remove_hop_headers(headers: &mut HeaderMap<HeaderValue>) {
    lazy_static! {
        static ref HOP_HEADERS: Vec<HeaderName> = [
            "Connection",
            "Keep-Alive",
            "Proxy-Authenticate",
            "Proxy-Authorization",
            "TE",
            "Trailers",
            "Transfer-Encoding",
            "Upgrade",
        ]
        .into_iter()
        .map(|h| h.parse().expect("Invalid header"))
        .collect();
    }

    for key in HOP_HEADERS.iter() {
        if let Entry::Occupied(entry) = headers.entry(key) {
            entry.remove_entry_mult();
        }
    }
}
