use super::{
    calc_total_length, parse_length, RespDecode, RespEncode, RespError, RespFrame, BUF_CAPACITY,
    CRLF_LEN,
};
use bytes::{Buf, BytesMut};

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq)]
pub struct RespArray(pub(crate) Option<Vec<RespFrame>>);

impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        match self.0 {
            None => b"*-1\r\n".to_vec(),
            Some(v) => {
                let mut buf = Vec::with_capacity(BUF_CAPACITY);
                buf.extend_from_slice(&format!("*{}\r\n", v.len()).into_bytes());
                for frame in v {
                    buf.extend_from_slice(&frame.encode());
                }
                buf
            }
        }
    }
}

/// *-1\r\n
impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    const FRAME_TYPE: &'static str = "RespArray";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;

        if len == -1 {
            buf.advance(end + CRLF_LEN);
            return Ok(RespArray(None));
        }
        let len = len as usize;
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
        if len == -1 {
            return Ok(4);
        }
        calc_total_length(buf, end, len as usize, Self::PREFIX)
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(Some(s.into()))
    }

    pub fn new_null_array() -> Self {
        RespArray(None)
    }

    pub fn len(&self) -> usize {
        match &self.0 {
            Some(v) => v.len(),
            None => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        let frame: RespFrame = RespArray(Some(array)).into();
        assert_eq!(
            String::from_utf8_lossy(&frame.encode()),
            "*3\r\n+foo\r\n+bar\r\n$3\r\nbaz\r\n"
        );
    }

    #[test]
    fn test_null_array() {
        let frame: RespFrame = RespArray::new_null_array().into();
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
