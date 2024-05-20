use crate::{
    backend::Backend,
    resp::{frame::RespFrame, RespArray, RespError, SimpleString},
};

use super::CommandExecuter;

#[derive(Debug)]
pub struct Echo(pub String);

impl CommandExecuter for Echo {
    fn execute(self, _backend: Backend) -> RespFrame {
        SimpleString::new(self.0).into()
    }
}
impl Echo {
    fn new(data: impl Into<String>) -> Self {
        Echo(data.into())
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = RespError;

    /// very ugly code
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        match value.len() {
            2 => match value.0 {
                Some(v) => match v.into_iter().nth(1) {
                    Some(v) => match v {
                        RespFrame::BulkString(s) => match s.0 {
                            Some(v) => Ok(Echo::new(String::from_utf8(v)?)),
                            None => Err(RespError::InvalidFrame(
                                "Expect a BulkString, but got a Null BulkString".into(),
                            )),
                        },
                        _ => Err(RespError::InvalidFrameType("Invalid frame type".into())),
                    },
                    None => Err(RespError::InvalidFrameLength(1)),
                },
                None => Err(RespError::InvalidFrame(
                    "Expect a Array length=2, but got a Null Array".into(),
                )),
            },
            _ => Err(RespError::InvalidFrameLength(value.len() as isize)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::{RespDecode, RespEncode};
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_echo_encode() {
        let echo = Echo::new("hello");
        let resp = echo.execute(Backend::default());
        let encoded = resp.encode();
        assert_eq!(encoded, b"+hello\r\n");
    }

    #[test]
    fn test_echo_decode() -> Result<()> {
        let mut buf = BytesMut::from("*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let echo = Echo::try_from(frame)?;
        assert_eq!(echo.0, "hello");
        Ok(())
    }
}
