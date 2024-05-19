use bytes::BytesMut;

use super::{extract_fixed_data, RespDecode, RespEncode, RespError};

impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { 't' } else { 'f' }).into_bytes()
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::frame::RespFrame;

    #[test]
    fn test_boolean_encode() {
        let frame: RespFrame = true.into();
        assert_eq!(frame.encode(), b"#t\r\n");

        let frame: RespFrame = false.into();
        assert_eq!(frame.encode(), b"#f\r\n");
    }
}
