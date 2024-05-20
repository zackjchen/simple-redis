use bytes::{Buf, BytesMut};
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use super::{
    calc_total_length, parse_length, simple_string::SimpleString, RespDecode, RespEncode,
    RespError, RespFrame, BUF_CAPACITY, CRLF_LEN,
};

/// Now only support string key which encode to SimpleString
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);

impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAPACITY);
        buf.extend_from_slice(&format!("%{}\r\n", self.0.len()).into_bytes());
        for (k, v) in self.0 {
            // buf.extend_from_slice(format!("+{}\r\n", k).as_bytes());
            buf.extend_from_slice(&SimpleString::new(k).encode());
            buf.extend_from_slice(&v.encode());
        }
        buf
    }
}

impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";
    const FRAME_TYPE: &'static str = "RespMap";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        let len = len as usize;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotCompleteFrame);
        }
        buf.advance(end + CRLF_LEN);
        let mut map = RespMap::new();
        for _ in 0..len {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            map.insert(key.0, value);
        }
        Ok(map)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        let len = len as usize;
        calc_total_length(buf, end, len, "%")
    }
}

impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

impl Default for RespMap {
    fn default() -> Self {
        RespMap::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::{BulkString, RespArray};
    use anyhow::Result;

    #[test]
    fn test_map_encode() {
        let mut map = RespMap::new();
        map.insert("hello".into(), SimpleString::new("world").into());
        map.insert("foo".into(), (-123456.789).into());
        let frame: RespFrame = map.into();
        assert_eq!(
            String::from_utf8_lossy(&frame.encode()),
            "%2\r\n+foo\r\n,-1.23456789e5\r\n+hello\r\n+world\r\n"
        );
    }

    #[test]
    fn test_map_decode() -> Result<()> {
        let buf =
            BytesMut::from("%3\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n+array\r\n*-1\r\n");
        let mut map = RespMap::new();
        map.insert("hello".into(), BulkString::new("world").into());
        map.insert("foo".into(), BulkString::new("bar").into());
        map.insert("array".into(), RespArray::new_null_array().into());

        let frame = RespMap::decode(&mut buf.clone())?;
        assert_eq!(frame, map);
        Ok(())
    }
}
