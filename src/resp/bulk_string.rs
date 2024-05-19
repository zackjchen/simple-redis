use std::ops::Deref;

use bytes::{Buf, BytesMut};

use super::{extract_fixed_data, parse_length, RespDecode, RespEncode, RespError, CRLF_LEN};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct BulkString(pub(crate) Vec<u8>);

impl Deref for BulkString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespNullBulkString;

impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(format!("${}\r\n", self.len()).as_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

impl RespEncode for RespNullBulkString {
    fn encode(self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}

/// $5\r\nhello\r\n
impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    const FRAME_TYPE: &'static str = "BulkString";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;
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
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}

impl RespDecode for RespNullBulkString {
    const PREFIX: &'static str = "$-1\r\n";
    const FRAME_TYPE: &'static str = "NullBulkString";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(data, Self::PREFIX, Self::FRAME_TYPE)?;
        Ok(RespNullBulkString)
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(5)
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

/// 这个写法需要好好记一下
impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
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
        let frame: RespFrame = RespNullBulkString.into();
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
        let frame = RespNullBulkString::decode(&mut buf)?;
        assert_eq!(frame, RespNullBulkString);
        Ok(())
    }
}
