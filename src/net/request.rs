use hyper::HeaderMap;
use tokio::{
    io::{self, AsyncReadExt, BufReader},
    net::TcpStream,
};

use super::{Method, Protocol};

pub struct Body<'a> {
    len: usize,
    reader: BufReader<&'a mut TcpStream>,
}

impl<'a> Body<'a> {
    pub fn new(len: usize, reader: BufReader<&'a mut TcpStream>) -> Self {
        Self { len, reader }
    }

    pub async fn array(mut self) -> io::Result<Vec<u8>> {
        // self.reader.take(limit)
        let mut result = Vec::with_capacity(self.len);
        const BUF_LEN: usize = 512;
        let mut buf = [0; BUF_LEN];
        let mut len = self.len;
        loop {
            let amt = self.reader.read_exact(&mut buf[..len.min(BUF_LEN)]).await?;
            result.extend_from_slice(&buf[..amt]);
            if len <= buf.len() {
                break;
            }
            len -= amt;
        }
        Ok(result)
    }

    pub async fn text(self) -> io::Result<String> {
        let result = self.array().await?;
        Ok(String::from_utf8_lossy(&result).to_string())
    }

    pub async fn plist(self) -> io::Result<plist::Value> {
        let mut reader = self.reader.take(self.len as u64);
        let mut result = Vec::with_capacity(self.len);
        reader.read_to_end(&mut result).await?;
        let value: plist::Value = plist::from_bytes(&result).expect("plist der error");
        Ok(value)
    }
}

pub struct Request {
    method: Method,
    protocol: Protocol,
    uri: String,
    body: Body<'_>,
    headers: HeaderMap,
}

impl Request {
    pub fn new(
        method: Method,
        protocol: Protocol,
        uri: String,
        body: Body,
        headers: HeaderMap,
    ) -> Self {
        Self {
            method,
            protocol,
            uri,
            body,
            headers,
        }
    }

    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn into_body(self) -> Body<'_> {
        self.body
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}
