use std::ops::Deref;

use bytes::BytesMut;

use super::{extract_simple_frame_data, RespDecode, RespEncode, RespError, CRLF_LEN};

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct SimpleString(pub(crate) String);

impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", *self).into_bytes()
    }
}

impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";
    const FRAME_TYPE: &'static str = "SimpleString";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[1..end]);
        Ok(SimpleString::new(s.to_string()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        Ok(end + CRLF_LEN)
    }
}

impl Deref for SimpleString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::{frame::RespFrame, SimpleString};
    use anyhow::Result;

    #[test]
    fn test_simple_string_encode() {
        let frame: RespFrame = SimpleString::new("OK").into();
        assert_eq!(frame.encode(), b"+OK\r\n");
    }

    #[test]
    fn test_simple_string_decode() -> Result<()> {
        let mut buf = BytesMut::from("+OK\r\n");
        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("OK"));

        buf.extend_from_slice(b"+hello\r");
        let frame = SimpleString::decode(&mut buf);
        assert_eq!(frame.unwrap_err(), RespError::NotCompleteFrame);
        buf.extend_from_slice(b"\n");
        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("hello"));
        Ok(())
    }

    #[test]
    fn test_simple_string_length() -> Result<()> {
        let buf = BytesMut::from("+OK\r\n");
        let frame = SimpleString::expect_length(&buf)?;
        assert_eq!(frame, 5);
        Ok(())
    }
}
