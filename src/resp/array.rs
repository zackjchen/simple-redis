use super::{
    calc_total_length, extract_fixed_data, parse_length, RespDecode, RespEncode, RespError,
    RespFrame, BUF_CAPACITY, CRLF_LEN,
};
use bytes::{Buf, BytesMut};
use std::ops::Deref;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespNullArray;

impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAPACITY);
        buf.extend_from_slice(&format!("*{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

impl RespEncode for RespNullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    const FRAME_TYPE: &'static str = "RespArray";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotCompleteFrame);
        }
        buf.advance(end + CRLF_LEN);
        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            let frame = RespFrame::decode(buf)?;
            frames.push(frame);
        }
        Ok(RespArray::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl RespDecode for RespNullArray {
    const PREFIX: &'static str = "*-1\r\n";
    const FRAME_TYPE: &'static str = "NullArray";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        Ok(RespNullArray)
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4)
    }
}
impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::{BulkString, RespMap, SimpleString};
    use anyhow::Result;

    #[test]
    fn test_array_encode() {
        let array: Vec<RespFrame> = vec![
            SimpleString::new("foo").into(),
            SimpleString::new("bar").into(),
            BulkString::new(b"baz".to_vec()).into(),
        ];
        let frame: RespFrame = RespArray(array).into();
        assert_eq!(
            String::from_utf8_lossy(&frame.encode()),
            "*3\r\n+foo\r\n+bar\r\n$3\r\nbaz\r\n"
        );
    }

    #[test]
    fn test_null_array() {
        let frame: RespFrame = RespNullArray.into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_array_decode() -> Result<()> {
        let mut buf = BytesMut::from(
            "*4\r\n+foo\r\n+bar\r\n$3\r\nbaz\r\n%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n",
        );
        let mut map = RespMap::new();
        map.insert("hello".into(), BulkString::new("world").into());
        map.insert("foo".into(), BulkString::new("bar").into());

        let array = RespArray::new(vec![
            SimpleString::new("foo").into(),
            SimpleString::new("bar").into(),
            BulkString::new("baz").into(),
            map.into(),
        ]);
        let frame = RespArray::decode(&mut buf.clone());
        assert_eq!(frame.unwrap_err(), RespError::NotCompleteFrame);
        buf.extend_from_slice("$3\r\nbar\r\n".as_bytes());
        assert_eq!(RespArray::decode(&mut buf.clone())?, array);
        Ok(())
    }
}
