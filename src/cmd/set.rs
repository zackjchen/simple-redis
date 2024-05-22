//! support sadd and sismember command

use super::CommandExecuter;
use crate::{
    backend::Backend,
    resp::{frame::RespFrame, RespArray, RespError},
};

#[derive(Debug)]
pub struct SAdd {
    key: String,
    members: Vec<RespFrame>,
}

#[derive(Debug)]
pub struct SisMember {
    key: String,
    member: RespFrame,
}

/// 虽然我手动给RespFrame实现了hash和Eq，但是我没有搞明白hash和Eq对f64作用，所以如果接收到f64类型的数据，直接返回0
impl CommandExecuter for SAdd {
    fn execute(self, backend: Backend) -> RespFrame {
        let len = self.members.len();
        let members = self
            .members
            .into_iter()
            .filter(|m| !matches!(m, RespFrame::Double(_)))
            .collect::<Vec<_>>();

        if members.len() != len {
            return RespFrame::Integer(0);
        }
        backend.sadd(&self.key, members);
        RespFrame::Integer(1)
    }
}

impl CommandExecuter for SisMember {
    fn execute(self, backend: Backend) -> RespFrame {
        let flag = backend.sismembers(&self.key, &self.member);

        RespFrame::Integer(flag as i64)
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = RespError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let mut iter = value
            .0
            .ok_or(RespError::InvalidFrame("sadd key cannot be Null".into()))?
            .into_iter()
            .skip(1);
        let key = match iter.next() {
            Some(RespFrame::BulkString(v)) => v.to_string(),
            _ => {
                return Err(RespError::InvalidFrameType(
                    "sadd key must be BulkString".into(),
                ))
            }
        };
        let members = iter.collect();
        Ok(SAdd { key, members })
    }
}

impl TryFrom<RespArray> for SisMember {
    type Error = RespError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        if value.len() != 3 {
            return Err(RespError::InvalidFrameLength(value.len() as isize));
        }
        let mut iter = value
            .0
            .ok_or(RespError::InvalidFrame("accept a empty array".into()))?
            .into_iter()
            .skip(1);

        let key = match iter.next() {
            Some(v) => match v {
                RespFrame::BulkString(k) => k.to_string(),
                _ => {
                    return Err(RespError::InvalidFrameType(
                        "cmd sismember key must be BulkString".into(),
                    ))
                }
            },
            None => {
                return Err(RespError::InvalidFrame(
                    "cmd sismember key cannot be Null".into(),
                ))
            }
        };
        let member = match iter.next() {
            Some(v) => v,
            None => {
                return Err(RespError::InvalidFrame(
                    "cmd sismember member cannot be Null".into(),
                ))
            }
        };
        Ok(SisMember { key, member })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backend::Backend;
    use crate::resp::frame::RespFrame;
    use crate::resp::{RespArray, SimpleString};
    #[test]
    fn test_sadd() {
        let backend = Backend::new();
        let cmd1 = SAdd {
            key: "key1".to_string(),
            members: vec![
                RespFrame::SimpleString(SimpleString::new("value1")),
                RespFrame::Integer(1),
                RespFrame::Double(1.0),
            ],
        };
        let cmd2 = SAdd {
            key: "key2".to_string(),
            members: vec![
                RespFrame::SimpleString(SimpleString::new("value2")),
                RespFrame::Integer(2),
                RespFrame::Array(RespArray::new_null_array()),
            ],
        };
        let resp = cmd1.execute(backend.clone());
        assert_eq!(resp, RespFrame::Integer(0));
        let resp = cmd2.execute(backend.clone());
        assert_eq!(resp, RespFrame::Integer(1));
    }

    #[test]
    fn test_sismember() {
        let backend = Backend::new();
        let cmd1 = SAdd {
            key: "key1".to_string(),
            members: vec![
                RespFrame::SimpleString(SimpleString::new("value1")),
                RespFrame::Integer(1),
            ],
        };
        cmd1.execute(backend.clone());
        let cmd2 = SisMember {
            key: "key1".to_string(),
            member: RespFrame::SimpleString(SimpleString::new("value1")),
        };
        let resp = cmd2.execute(backend.clone());
        assert_eq!(resp, RespFrame::Integer(1));

        let cmd2 = SisMember {
            key: "key1".to_string(),
            member: RespFrame::SimpleString(SimpleString::new("zack")),
        };
        let resp = cmd2.execute(backend.clone());
        assert_eq!(resp, RespFrame::Integer(0));
    }
}
