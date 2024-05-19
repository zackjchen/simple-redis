use bytes::BytesMut;

use super::{extract_fixed_data, RespDecode, RespEncode, RespError};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespNull;

impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
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

#[cfg(test)]
mod test {

    use super::*;
    use anyhow::Result;
    #[test]
    fn test_null_decode() -> Result<()> {
        let mut buf = BytesMut::from("_\r\n");
        let frame = RespNull::decode(&mut buf)?;
        assert_eq!(frame, RespNull);
        Ok(())
    }
}
