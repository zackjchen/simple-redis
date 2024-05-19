use super::{extract_simple_frame_data, RespEncode};
use bytes::BytesMut;

use super::{RespDecode, RespError, CRLF_LEN};

impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        format!(",{:+e}\r\n", self).into_bytes()
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
#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::frame::RespFrame;
    use anyhow::Result;

    #[test]
    fn test_double_encode() {
        let frame: RespFrame = 5.21.into();
        assert_eq!(String::from_utf8_lossy(&frame.encode()), ",+5.21e0\r\n");

        let frame = RespFrame::from(-5.21);
        assert_eq!(frame.encode(), b",-5.21e0\r\n");

        let frame: RespFrame = 1.23456e+8.into();
        assert_eq!(String::from_utf8_lossy(&frame.encode()), ",+1.23456e8\r\n");

        let frame: RespFrame = (-1.23456e-8).into();
        assert_eq!(frame.encode(), b",-1.23456e-8\r\n");
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
}
