use std::ops::Deref;

use bytes::{Buf, BytesMut};

use super::{calc_total_length, parse_length, CRLF_LEN};
use super::{RespDecode, RespEncode, RespError, RespFrame, BUF_CAPACITY};
#[derive(Debug, PartialEq, PartialOrd, Clone, Eq)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

impl RespEncode for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAPACITY);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

impl RespDecode for RespSet {
    const PREFIX: &'static str = "~";
    const FRAME_TYPE: &'static str = "RespSet";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // let prefix = "~";
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        let len = len as usize;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotCompleteFrame);
        }
        buf.advance(end + CRLF_LEN);
        let mut set = Vec::with_capacity(len);

        for _ in 0..len {
            let frame = RespFrame::decode(buf)?;
            set.push(frame);
        }
        Ok(RespSet::new(set))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        let len = len as usize;
        calc_total_length(buf, end, len, "~")
    }
}

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod test {

    use crate::resp::{
        frame::RespFrame, BulkString, RespArray, RespDecode, RespEncode, RespSet, SimpleString,
    };
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_set_encode() {
        let arr = vec![
            SimpleString::new("foo").into(),
            SimpleString::new("bar").into(),
            RespFrame::Double(-5.21),
            RespArray(Some(vec![SimpleString::new("hello").into()])).into(),
        ];
        let frame: RespFrame = RespArray(Some(arr)).into();
        assert_eq!(
            frame.encode(),
            b"*4\r\n+foo\r\n+bar\r\n,-5.21e0\r\n*1\r\n+hello\r\n"
        );
    }

    #[test]
    fn test_set_decode() -> Result<()> {
        let buf = BytesMut::from("~3\r\n+foo\r\n+bar\r\n$3\r\nbaz\r\n");
        let set: Vec<RespFrame> = vec![
            SimpleString::new("foo").into(),
            SimpleString::new("bar").into(),
            BulkString::new("baz").into(),
        ];
        let frame = RespSet::decode(&mut buf.clone())?;
        assert_eq!(frame, RespSet::new(set));
        Ok(())
    }
}
