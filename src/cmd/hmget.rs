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
