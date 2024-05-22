use super::{parse_length, RespDecode, RespEncode, RespError, CRLF_LEN};
use bytes::{Buf, BytesMut};
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq)]
pub struct BulkString(pub(crate) Option<Vec<u8>>);

impl Deref for BulkString {
    type Target = Option<Vec<u8>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// #[derive(Debug, PartialEq, PartialOrd, Clone)]
// pub struct RespNullBulkString;

impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        match self.as_deref() {
            Some(v) => {
                let mut buf = Vec::with_capacity(v.len() + 16);
                buf.extend_from_slice(format!("${}\r\n", v.len()).as_bytes());
                buf.extend_from_slice(v);
                buf.extend_from_slice(b"\r\n");
                buf
            }
            None => "$-1\r\n".as_bytes().to_vec(),
        }
    }
}

/// $5\r\nhello\r\n
/// $-1\r\n
impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    const FRAME_TYPE: &'static str = "BulkString";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        if len == -1 {
            buf.advance(end + CRLF_LEN);
            return Ok(BulkString(None));
        }
        let len = len as usize;
        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotCompleteFrame);
        }
        buf.advance(end + CRLF_LEN);
        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data[..len].to_vec()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        if len == -1 {
            return Ok(end + CRLF_LEN);
        }
        Ok(end + CRLF_LEN + len as usize + CRLF_LEN)
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(Some(s.into()))
    }
    pub fn new_null_string() -> Self {
        BulkString(None)
    }
}

/// 这个写法需要好好记一下
impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(Some(s.to_vec()))
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(Some(s.as_bytes().to_vec()))
    }
}

impl Display for BulkString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.as_deref() {
            Some(v) => write!(f, "{}", String::from_utf8_lossy(v)),
            None => write!(f, "null"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::{frame::RespFrame, BulkString};
    use anyhow::Result;

    #[test]
    fn test_bulk_string_encode() {
        let frame: RespFrame = BulkString::new(b"foobar".to_vec()).into();
        assert_eq!(frame.encode(), b"$6\r\nfoobar\r\n");
    }

    #[test]
    fn test_null_bulk_string_encode() {
        let frame: RespFrame = BulkString::new_null_string().into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::from("$5\r\nhello\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello".to_vec()));

        buf.extend_from_slice(b"$5\r\nhello");
        let frame = BulkString::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotCompleteFrame);
        buf.extend_from_slice(b"\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello".to_vec()));
        Ok(())
    }

    #[test]
    fn test_bulk_string_length() -> Result<()> {
        let buf = BytesMut::from("$5\r\nhello\r\n");
        let frame = BulkString::expect_length(&buf)?;
        assert_eq!(frame, 11);
        Ok(())
    }
    #[test]
    fn test_null_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::from("$-1\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new_null_string());
        Ok(())
    }
}
