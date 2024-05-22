use crate::{
    backend::Backend,
    resp::{array::RespArray, frame::RespFrame, BulkString},
};

use super::{
    extract_args, validate_command, CommandError, CommandExecuter, HGet, HGetAll, HSet, RESP_OK,
};

impl CommandExecuter for HGet {
    fn execute(self, backend: Backend) -> RespFrame {
        backend
            .hget(&self.key, &self.field)
            .unwrap_or(RespFrame::BulkString(BulkString::new_null_string()))
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

        match map {
            Some(hmap) => {
                let mut data = Vec::with_capacity(hmap.len());
                for v in hmap.iter() {
                    let key = v.key().to_owned();
                    data.push((key, v.value().clone()));
                }

                data.sort_by(|a, b| a.0.cmp(&b.0));

                let ret = data
                    .into_iter()
                    .flat_map(|(k, v)| vec![BulkString::new(k).into(), v])
                    .collect::<Vec<RespFrame>>();

                RespArray::new(ret).into()
            }
            None => RespArray::new([]).into(),
        }
    }
}

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hget"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => Ok(HGet {
                key: String::from_utf8(key.0.unwrap())?,
                field: String::from_utf8(field.0.unwrap())?,
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
                key: String::from_utf8(key.0.unwrap())?,
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
                    key: String::from_utf8(key.0.unwrap())?,
                    field: String::from_utf8(field.0.unwrap())?,
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
        resp::{array::RespArray, bulk_string::BulkString, frame::RespFrame, RespDecode},
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

        let resp_array: Vec<RespFrame> = vec![
            BulkString::new("field1").into(),
            BulkString::new("value1").into(),
            BulkString::new("field2").into(),
            BulkString::new("value2").into(),
        ];

        assert_eq!(resp, RespFrame::Array(RespArray::new(resp_array)));

        let mut hgetall2 = BytesMut::from("*2\r\n$7\r\nhgetall\r\n$4\r\nkey2\r\n");
        let hgetall2 = RespArray::decode(&mut hgetall2)?;
        let hgetall2 = HGetAll::try_from(hgetall2)?;
        let resp2 = hgetall2.execute(backend.clone());
        println!("{:?}", resp2);
        assert_eq!(resp2, RespArray::new([]).into());

        Ok(())
    }
}
