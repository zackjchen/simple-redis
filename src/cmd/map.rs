use crate::{
    backend::Backend,
    cmd::{CommandError, Get},
    resp::{array::RespArray, frame::RespFrame, null::RespNull},
};

use super::{extract_args, validate_command, CommandExecuter, Set, RESP_OK};

impl CommandExecuter for Get {
    fn execute(self, backend: Backend) -> RespFrame {
        backend.get(&self.key).unwrap_or(RespFrame::Null(RespNull))
    }
}

impl CommandExecuter for Set {
    fn execute(self, backend: Backend) -> RespFrame {
        backend
            .set(&self.key, self.value)
            .unwrap_or(RESP_OK.clone())
    }
}

impl TryFrom<RespArray> for Get {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["get"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        let key = match args.next() {
            Some(RespFrame::BulkString(key)) => String::from_utf8(key.0.unwrap())?,
            _ => {
                return Err(CommandError::InvalidArgument(
                    "key must be a bulk string".to_string(),
                ))
            }
        };
        Ok(Get { key })
    }
}

impl TryFrom<RespArray> for Set {
    type Error = CommandError;

    /// validate 时有key和value两个参数
    /// extract_args 时要skep掉第一个set cmd
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["set"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        let (key, val) = match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(value)) => {
                (String::from_utf8(key.0.unwrap())?, value)
            }
            _ => {
                return Err(CommandError::InvalidArgument(
                    "key must be a bulk string".to_string(),
                ))
            }
        };
        Ok(Set { key, value: val })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        cmd::{Get, Set},
        resp::{array::RespArray, bulk_string::BulkString, frame::RespFrame, RespDecode},
    };
    use bytes::BytesMut;
    use std::vec;

    #[test]
    fn test_get_try_from_resp_array() {
        let frame = RespArray::new(vec![
            BulkString(Some(b"get".to_vec())).into(),
            BulkString(Some(b"key".to_vec())).into(),
        ]);
        let get_cmd = Get::try_from(frame).unwrap();
        assert_eq!(get_cmd.key, "key");
    }

    #[test]
    fn test_set_try_from_resp_array() -> anyhow::Result<()> {
        let mut buf = BytesMut::from("*3\r\n$3\r\nset\r\n$3\r\nkey\r\n$5\r\nvalue\r\n");
        let frame = RespArray::decode(&mut buf).unwrap();
        let set_cmd = Set::try_from(frame)?;
        assert_eq!(set_cmd.key, "key");
        assert_eq!(
            set_cmd.value,
            RespFrame::BulkString(BulkString(Some(b"value".to_vec())))
        );
        Ok(())
    }
}
