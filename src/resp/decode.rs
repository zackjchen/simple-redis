use bytes::{Buf, BytesMut};

use super::{
    calc_total_length, extract_fixed_data, extract_simple_frame_data, parse_length, BulkString,
    RespArray, RespFrame, RespMap, RespNull, RespNullArray, RespNullBulkString, RespSet,
    SimpleError, SimpleString, CRLF_LEN,
};
use crate::resp::{RespDecode, RespError};

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
            Some(b'$') => match RespNullBulkString::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(RespError::NotCompleteFrame) => Err(RespError::NotCompleteFrame),
                Err(_) => {
                    let frame = BulkString::decode(buf)?;
                    Ok(frame.into())
                }
            },
            Some(b'*') => match RespNullArray::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(RespError::NotCompleteFrame) => Err(RespError::NotCompleteFrame),
                Err(_) => {
                    let frame = RespArray::decode(buf)?;
                    Ok(frame.into())
                }
            },
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

impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    const FRAME_TYPE: &'static str = "SimpleError";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[1..end]);
        Ok(SimpleError::new(s.to_string()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        Ok(end + CRLF_LEN)
    }
}

// :1000\r\n
impl RespDecode for i64 {
    const PREFIX: &'static str = ":";
    const FRAME_TYPE: &'static str = "i64";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[1..end]);
        let num = s.parse::<i64>()?;
        Ok(num)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        Ok(end + CRLF_LEN)
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

impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";
    const FRAME_TYPE: &'static str = "Null";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(data, Self::PREFIX, Self::FRAME_TYPE)?;
        Ok(RespNull)
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(3)
    }
}

impl RespDecode for bool {
    const PREFIX: &'static str = "#";
    const FRAME_TYPE: &'static str = "Bool";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_fixed_data(buf, Self::PREFIX, Self::FRAME_TYPE) {
            Ok(_) => Ok(true),
            Err(RespError::NotCompleteFrame) => Err(RespError::NotCompleteFrame),
            Err(_) => match extract_fixed_data(buf, Self::PREFIX, Self::FRAME_TYPE) {
                Ok(_) => Ok(false),
                Err(e) => Err(e),
            },
        }
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4)
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

// b",-1.23456e-8\r\n"
impl RespDecode for f64 {
    const PREFIX: &'static str = ",";
    const FRAME_TYPE: &'static str = "f64";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[1..end]);
        let num = s.parse::<f64>()?;

        Ok(num)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX, Self::FRAME_TYPE)?;
        Ok(end + CRLF_LEN)
    }
}
impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";
    const FRAME_TYPE: &'static str = "RespMap";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;
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
        calc_total_length(buf, end, len, "%")
    }
}

impl RespDecode for RespSet {
    const PREFIX: &'static str = "~";
    const FRAME_TYPE: &'static str = "RespSet";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // let prefix = "~";
        let (end, len) = parse_length(buf, Self::PREFIX, Self::FRAME_TYPE)?;
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
        calc_total_length(buf, end, len, "~")
    }
}

#[cfg(test)]
mod tests {
    use crate::resp::{
        BulkString, RespArray, RespDecode, RespError, RespFrame, RespMap, RespNull, RespNullArray,
        RespNullBulkString, RespSet, SimpleError, SimpleString,
    };
    use anyhow::Result;
    use bytes::BytesMut;
    #[test]
    fn test_simple_string_decode() -> anyhow::Result<()> {
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
    fn test_simple_string_length() -> anyhow::Result<()> {
        let buf = BytesMut::from("+OK\r\n");
        let frame = SimpleString::expect_length(&buf)?;
        assert_eq!(frame, 5);
        Ok(())
    }
    #[test]
    fn test_simple_error_decode() -> Result<()> {
        let mut buf = BytesMut::from("-Error message\r\n");
        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(frame, SimpleError::new("Error message"));
        Ok(())
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

    #[test]
    fn test_null_decode() -> Result<()> {
        let mut buf = BytesMut::from("_\r\n");
        let frame = RespNull::decode(&mut buf)?;
        assert_eq!(frame, RespNull);
        Ok(())
    }

    #[test]
    fn test_double_decode() -> Result<()> {
        let mut buf = BytesMut::from(",1.23456e-8\r\n");
        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 1.23456e-8);

        let mut buf = BytesMut::from(",1234.56\r\n");
        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 1.23456e3);
        Ok(())
    }

    #[test]
    fn test_map_decode() -> Result<()> {
        let buf =
            BytesMut::from("%3\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n+array\r\n*-1\r\n");
        let mut map = RespMap::new();
        map.insert("hello".into(), BulkString::new("world").into());
        map.insert("foo".into(), BulkString::new("bar").into());
        map.insert("array".into(), RespNullArray.into());

        let frame = RespMap::decode(&mut buf.clone())?;
        assert_eq!(frame, map);
        Ok(())
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
