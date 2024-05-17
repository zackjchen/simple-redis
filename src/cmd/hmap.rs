use crate::{
    backend::Backend,
    resp::{RespArray, RespFrame, RespMap, RespNull},
};

use super::{
    extract_args, validate_command, CommandError, CommandExecuter, HGet, HGetAll, HSet, RESP_OK,
};

impl CommandExecuter for HGet {
    fn execute(self, backend: Backend) -> RespFrame {
        backend
            .hget(&self.key, &self.field)
            .unwrap_or(RespFrame::Null(RespNull))
    }
}

impl CommandExecuter for HSet {
    fn execute(self, backend: Backend) -> RespFrame {
        backend
            .hset(&self.key, &self.field, self.value)
            .unwrap_or(RESP_OK.clone())
    }
}
impl CommandExecuter for HGetAll {
    fn execute(self, backend: Backend) -> RespFrame {
        let map = backend.hget_all(&self.key);
        let mut resp_map = RespMap::new();

        if let Some(map) = map {
            for v in map.iter() {
                resp_map.insert(v.key().clone(), v.value().clone());
            }
        }
        resp_map.into()
    }
}

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hget"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => Ok(HGet {
                key: String::from_utf8(key.0)?,
                field: String::from_utf8(field.0)?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "HGet Command, key and field must be bulk string".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HGetAll {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hgetall"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(HGetAll {
                key: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "HGetAll Command, key must be bulk string".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HSet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hset"], 3)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field)), Some(value)) => {
                Ok(HSet {
                    key: String::from_utf8(key.0)?,
                    field: String::from_utf8(field.0)?,
                    value,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "HSet Command, key, field and value must be bulk string".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        backend::Backend,
        cmd::{CommandExecuter, HGet, HGetAll, HSet, RESP_OK},
        resp::{BulkString, RespArray, RespDecode, RespFrame, RespMap},
    };
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_hget_try_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::from("*3\r\n$4\r\nhget\r\n$3\r\nkey\r\n$5\r\nfield\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let hget_cmd = HGet::try_from(frame).unwrap();
        assert_eq!(hget_cmd.key, "key");
        assert_eq!(hget_cmd.field, "field");
        Ok(())
    }
    #[test]
    fn test_hgetall_try_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::from("*2\r\n$7\r\nhgetall\r\n$3\r\nkey\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let hgetall_cmd = HGetAll::try_from(frame).unwrap();
        assert_eq!(hgetall_cmd.key, "key");
        Ok(())
    }

    #[test]
    fn test_hset_try_from_resp_array() -> Result<()> {
        let mut buf =
            BytesMut::from("*4\r\n$4\r\nhset\r\n$3\r\nkey\r\n$5\r\nfield\r\n$5\r\nvalue\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let hset_cmd = HSet::try_from(frame)?;
        assert_eq!(hset_cmd.key, "key");
        assert_eq!(hset_cmd.field, "field");
        assert_eq!(
            hset_cmd.value,
            RespFrame::BulkString(BulkString::new("value"))
        );
        Ok(())
    }

    #[test]
    fn test_hset_hgetall() -> Result<()> {
        let backend = Backend::new();
        let mut hset1 =
            BytesMut::from("*4\r\n$4\r\nhset\r\n$3\r\nkey\r\n$6\r\nfield1\r\n$6\r\nvalue1\r\n");
        let mut hset2 =
            BytesMut::from("*4\r\n$4\r\nhset\r\n$3\r\nkey\r\n$6\r\nfield2\r\n$6\r\nvalue2\r\n");
        let hset1 = RespArray::decode(&mut hset1)?;
        let hset2 = RespArray::decode(&mut hset2)?;
        let hset1 = HSet::try_from(hset1)?;
        let hset2 = HSet::try_from(hset2)?;
        assert_eq!(hset1.execute(backend.clone()), RESP_OK.clone());
        assert_eq!(hset2.execute(backend.clone()), RESP_OK.clone());

        let mut hgetall = BytesMut::from("*2\r\n$7\r\nhgetall\r\n$3\r\nkey\r\n");
        let hgetall = RespArray::decode(&mut hgetall)?;
        let hgetall = HGetAll::try_from(hgetall)?;
        let resp = hgetall.execute(backend.clone());

        let mut resp_map = RespMap::new();
        resp_map.insert(
            "field1".to_string(),
            RespFrame::BulkString(BulkString::new("value1")),
        );
        resp_map.insert(
            "field2".to_string(),
            RespFrame::BulkString(BulkString::new("value2")),
        );
        assert_eq!(resp, RespFrame::Map(resp_map));

        let mut hgetall2 = BytesMut::from("*2\r\n$7\r\nhgetall\r\n$4\r\nkey2\r\n");
        let hgetall2 = RespArray::decode(&mut hgetall2)?;
        let hgetall2 = HGetAll::try_from(hgetall2)?;
        let resp2 = hgetall2.execute(backend.clone());
        println!("{:?}", resp2);
        assert_eq!(resp2, RespFrame::Map(RespMap::new()));

        Ok(())
    }
}
