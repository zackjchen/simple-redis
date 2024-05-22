use crate::{
    backend::Backend,
    resp::{frame::RespFrame, RespArray, RespError},
};

use super::{CommandExecuter, HGet};

#[derive(Debug)]
pub struct HmGet {
    key: String,
    fields: Vec<HGet>,
}

impl CommandExecuter for HmGet {
    fn execute(self, backend: Backend) -> RespFrame {
        let mut resps = Vec::new();
        for hget in self.fields {
            let resp = hget.execute(backend.clone());
            resps.push(resp);
        }
        RespArray::new(resps).into()
    }
}

impl TryFrom<RespArray> for HmGet {
    type Error = RespError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        if value.len() < 3 {
            return Err(RespError::InvalidFrameLength(value.len() as isize));
        }
        let mut args = value
            .0
            .ok_or(RespError::InvalidFrame("RespArray is None".to_string()))?
            .into_iter()
            .skip(1); // 第一个是hmget cmd，在外面以及验证过了，这里skip

        let key = match args.next() {
            Some(v) => match v {
                RespFrame::BulkString(k) => String::from_utf8(
                    k.0.ok_or(RespError::InvalidFrame("hmset key cannot be Null".into()))?,
                )?,
                _ => {
                    return Err(RespError::InvalidFrame(
                        "hmset key must be a bulk string".to_string(),
                    ))
                }
            },
            None => {
                return Err(RespError::InvalidFrame(
                    "hmset key cannot be Null".to_string(),
                ))
            }
        };
        let mut fields = vec![];
        for frame in args {
            match frame {
                RespFrame::BulkString(f) => {
                    let field = String::from_utf8(
                        f.0.ok_or(RespError::InvalidFrame("hmset field cannot be Null".into()))?,
                    )?;
                    fields.push(HGet {
                        key: key.clone(),
                        field,
                    });
                }
                _ => {
                    return Err(RespError::InvalidFrame(
                        "hmset field must be a bulk string".to_string(),
                    ))
                }
            }
        }
        Ok(HmGet { key, fields })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        backend,
        cmd::{Command, CommandExecuter},
        resp::{frame::RespFrame, BulkString, RespArray, RespDecode},
    };
    use anyhow::Result;
    use bytes::BytesMut;
    #[test]
    fn test_hmget() -> Result<()> {
        let backend = backend::Backend::new();
        let hmget = "*4\r\n$5\r\nhmget\r\n$3\r\nkey\r\n$6\r\nfield1\r\n$5\r\nfield\r\n".as_bytes();
        let hset1 = "*4\r\n$4\r\nhset\r\n$3\r\nkey\r\n$6\r\nfield1\r\n$6\r\nvalue1\r\n".as_bytes();
        let hset2 = "*4\r\n$4\r\nhset\r\n$3\r\nkey\r\n$6\r\nfield2\r\n$6\r\nvalue2\r\n".as_bytes();
        let frame1 = RespFrame::decode(&mut BytesMut::from(hset1))?;
        let frame2 = RespFrame::decode(&mut BytesMut::from(hset2))?;
        let frame = RespFrame::decode(&mut BytesMut::from(hmget))?;
        let hset1 = Command::try_from(frame1)?;
        let hset2 = Command::try_from(frame2)?;
        let hmget = Command::try_from(frame)?;

        hset1.execute(backend.clone());
        hset2.execute(backend.clone());
        let resp = hmget.execute(backend.clone());
        assert_eq!(
            resp,
            RespFrame::Array(RespArray::new(vec![
                RespFrame::BulkString(BulkString::new(b"value1")),
                RespFrame::BulkString(BulkString::new_null_string()),
            ]))
        );

        Ok(())
    }
}
