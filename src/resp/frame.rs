use std::hash::Hash;

use bytes::BytesMut;
use enum_dispatch::enum_dispatch;

use crate::resp::{
    array::RespArray, bulk_string::BulkString, map::RespMap, null::RespNull, set::RespSet,
    simple_error::SimpleError, simple_string::SimpleString,
};

use super::{RespDecode, RespError};
/// SampleString和SimpleError都包裹一下，否则实现trait的时候没办法区分
#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum RespFrame {
    SimpleString(SimpleString),
    SimpleError(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    // NullBulkString(RespNullBulkString),
    Array(RespArray),
    Null(RespNull),
    // NullArray(RespNullArray),
    Boolean(bool),
    // f64不支持hash
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

/// 这里强行加Eq，遇到f64类型应该会报错
impl Eq for RespFrame {}
impl Hash for RespFrame {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

/// 我觉得encode的实现放在一起方便我读，我先不refactor成单独的文件
impl RespDecode for RespFrame {
    const PREFIX: &'static str = "";

    const FRAME_TYPE: &'static str = "RespFrame";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'-') => {
                let frame = SimpleError::decode(buf)?;
                Ok(frame.into())
            }
            Some(b':') => {
                let frame = i64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'$') => {
                let frame = BulkString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'*') => {
                let frame = RespArray::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'_') => {
                let frame = RespNull::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'#') => {
                let frame = bool::decode(buf)?;
                Ok(frame.into())
            }
            Some(b',') => {
                let frame = f64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'%') => {
                let frame = RespMap::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'~') => {
                let frame = RespSet::decode(buf)?;
                Ok(frame.into())
            }
            None => Err(RespError::NotCompleteFrame),
            _ => todo!(),
        }
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => SimpleString::expect_length(buf),
            Some(b'-') => SimpleError::expect_length(buf),
            Some(b':') => i64::expect_length(buf),
            // 这里BulkString::expect_length包含了NullBulkString的情况
            Some(b'$') => BulkString::expect_length(buf),
            // 这里RespArray::expect_length包含了NullArray的情况
            Some(b'*') => RespArray::expect_length(buf),
            Some(b'_') => RespNull::expect_length(buf),
            Some(b'#') => bool::expect_length(buf),
            Some(b',') => f64::expect_length(buf),
            Some(b'%') => RespMap::expect_length(buf),
            Some(b'~') => RespSet::expect_length(buf),
            _ => Err(RespError::NotCompleteFrame),
        }
    }
}
impl From<&str> for RespFrame {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string()).into()
    }
}

impl From<&[u8]> for RespFrame {
    fn from(s: &[u8]) -> Self {
        BulkString(Some(s.to_vec())).into()
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString(Some(s.to_vec())).into()
    }
}
#[cfg(test)]
mod test {}
