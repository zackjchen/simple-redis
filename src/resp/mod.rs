#![allow(dead_code)]

pub mod decode;
pub mod encode;
use bytes::{Buf, BytesMut};
use enum_dispatch::enum_dispatch;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};
use thiserror::Error;

const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();
const BUF_CAPACITY: usize = 4096;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RespError {
    #[error("Invalid RESP frame: {0}")]
    InvalidFrame(String),
    #[error("Invalid RESP frame type: {0}")]
    InvalidFrameType(String),
    #[error("Invalid RESP frame length: {0}")]
    InvalidFrameLength(isize),
    #[error("Frame is not complete")]
    NotCompleteFrame,
    #[error("parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

pub trait RespDecode: Sized {
    const PREFIX: &'static str;
    const FRAME_TYPE: &'static str;
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

/// SampleString和SimpleError都包裹一下，否则实现trait的时候没办法区分
#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum RespFrame {
    SimpleString(SimpleString),
    SimpleError(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    NullBulkString(RespNullBulkString),
    Array(RespArray),
    Null(RespNull),
    NullArray(RespNullArray),
    Boolean(bool),
    // f64不支持hash
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct SimpleString(pub(crate) String);

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct SimpleError(pub(crate) String);

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespNull;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespNullArray;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespNullBulkString;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct BulkString(pub(crate) Vec<u8>);

/// Now only support string key which encode to SimpleString
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

impl Deref for SimpleString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SimpleError {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
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

impl Deref for RespSet {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}
impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}
impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
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

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}
impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

fn find_crlf(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|window| window == CRLF)
}
/// 这个写法需要好好记一下
impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}

/// %2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n
fn calc_total_length(buf: &[u8], end: usize, len: usize, prefix: &str) -> Result<usize, RespError> {
    let mut total = end + CRLF_LEN;
    let mut data = &buf[total..];
    match prefix {
        "*" | "~" => {
            for _ in 0..len {
                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        "%" => {
            for _ in 0..len {
                let len = SimpleString::expect_length(data)?;
                data = &data[len..];
                total += len;
                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        _ => Ok(len + CRLF_LEN),
    }
}

fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &str,
    expect_type: &str,
) -> Result<(), RespError> {
    if buf.len() < expect.len() {
        return Err(RespError::NotCompleteFrame);
    }

    if !buf.starts_with(expect.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: {}, got: {:?}",
            expect_type, buf
        )));
    }
    buf.advance(expect.len());

    Ok(())
}

/// 抽取prefix到第一个\r\n之间的数据, 返回\r的下标，没有split
fn extract_simple_frame_data(
    buf: &[u8],
    prefix: &str,
    frame_type: &str,
) -> Result<usize, RespError> {
    if buf.len() < 3 {
        return Err(RespError::NotCompleteFrame);
    };
    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: {}({}), got: {:?}",
            frame_type,
            prefix,
            String::from_utf8_lossy(buf)
        )));
    };

    // search for \r\n, 这一串用API更好
    // let mut end = 0;
    // for i in 0..buf.len() - 1 {
    //     if buf[i] == b'\r' && buf[i + 1] == b'\n' {
    //         end = i;
    //         break;
    //     }
    // }
    // if end == 0 {
    //     return Err(RespError::NotCompleteFrame);
    // }
    let end = find_crlf(buf).ok_or(RespError::NotCompleteFrame)?;

    Ok(end)
}

/// 去除prefix到第一个\r\n中的数据，解析成usize作为长度，返回\r的下标和长度
fn parse_length(buf: &[u8], prefix: &str, frame_type: &str) -> Result<(usize, usize), RespError> {
    let end = extract_simple_frame_data(buf, prefix, frame_type)?;
    let s = String::from_utf8_lossy(&buf[prefix.len()..end]);
    // 这里为了保证当空数组的时候，返回0
    let len = s.parse::<isize>()?;
    let len = if len < 0 { 0 } else { len as usize };
    Ok((end, len))
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use bytes::BytesMut;

    use crate::resp::find_crlf;

    use super::{RespDecode, RespFrame};

    #[test]
    fn test_find_crlf() {
        let buf = b"hello\r\nworld";
        assert_eq!(find_crlf(buf), Some(5));
    }

    #[test]
    fn test_calc_total_length() -> Result<()> {
        let mut buf = BytesMut::from("%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");
        let len = RespFrame::expect_length(buf.as_ref())?;
        assert_eq!(len, 38);
        buf.clear();
        buf.extend_from_slice("*5\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n".as_bytes());
        let len = RespFrame::expect_length(buf.as_ref())?;
        assert_eq!(len, 76);
        Ok(())
    }
}
