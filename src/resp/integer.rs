use bytes::BytesMut;

use super::{extract_simple_frame_data, RespDecode, RespEncode, RespError, CRLF_LEN};

impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        // 如果是负数，format自己会加上负号，正数会省略所以补上
        let sign = if self < 0 { "" } else { "+" };
        format!(":{}{}\r\n", sign, self).into_bytes()
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::frame::RespFrame;

    #[test]
    fn test_integer_encode() {
        let frame: RespFrame = 123.into();
        assert_eq!(frame.encode(), b":+123\r\n");

        let frame: RespFrame = (-123).into();
        assert_eq!(frame.encode(), b":-123\r\n");
    }
}
