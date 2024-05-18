#![allow(dead_code)]
use crate::{
    backend::Backend,
    resp::{RespArray, RespFrame, SimpleString},
};
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;
mod hmap;
mod map;
lazy_static! {
    static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}
#[enum_dispatch]
pub trait CommandExecuter {
    fn execute(self, backend: Backend) -> RespFrame;
}
#[derive(Debug)]
#[enum_dispatch(CommandExecuter)]
pub enum Command {
    Get(Get),
    Set(Set),
    HSet(HSet),
    HGet(HGet),
    HGetAll(HGetAll),
    Unrecongnized(Unrecongnized),
}
#[derive(Debug)]
pub struct Unrecongnized;
impl CommandExecuter for Unrecongnized {
    fn execute(self, _backend: Backend) -> RespFrame {
        RESP_OK.clone()
    }
}

#[derive(Debug)]
pub struct Get {
    key: String,
}

#[derive(Debug)]
pub struct Set {
    key: String,
    value: RespFrame,
}

/// HSET key field value：将哈希表 key 中的字段 field 的值设为 value。
/// 如果 key 不存在，一个新的哈希表被创建并进行 HSET 操作。如果字段 field 已经存在于哈希表中，旧值将被覆盖。
/// eg. hset key field value
#[derive(Debug)]
pub struct HSet {
    key: String,
    field: String,
    value: RespFrame,
}

/// HGET key field：获取存储在哈希表 key 中指定字段 field 的值。如果 key 或 field 不存在，返回 nil。
#[derive(Debug)]
pub struct HGet {
    key: String,
    field: String,
}

#[derive(Debug)]
pub struct HGetAll {
    key: String,
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("RespError: {0}")]
    RespError(#[from] crate::resp::RespError),
    #[error("Utf8Error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
impl TryFrom<RespArray> for Command {
    type Error = CommandError;

    fn try_from(frames: RespArray) -> Result<Self, Self::Error> {
        match frames.first() {
            Some(RespFrame::BulkString(frame)) => match frame.as_ref() {
                b"get" => Ok(Get::try_from(frames)?.into()),
                b"set" => Ok(Set::try_from(frames)?.into()),
                b"hset" => Ok(HSet::try_from(frames)?.into()),
                b"hget" => Ok(HGet::try_from(frames)?.into()),
                b"hgetall" => Ok(HGetAll::try_from(frames)?.into()),
                _ => Ok(Unrecongnized.into()),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must be a BulkString as the first arguments".to_string(),
            )),
        }
    }
}
impl TryFrom<RespFrame> for Command {
    type Error = CommandError;

    fn try_from(value: RespFrame) -> Result<Self, Self::Error> {
        match value {
            RespFrame::Array(frames) => Command::try_from(frames),
            _ => Err(CommandError::InvalidArgument(
                "Parse RespFrame to Command error, Command must be an RespArray".to_string(),
            )),
        }
    }
}

/// 这是用来解析client的命令的,
/// 发过来的一定都是BulkString的数组
fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have {} arguments",
            names.join(" "),
            n_args
        )));
    }
    if let Some((i, name)) = names.iter().enumerate().next() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.as_ref().to_ascii_lowercase() == name.as_bytes() {
                    return Ok(());
                } else {
                    return Err(CommandError::InvalidCommand(
                        "Invalid Get command".to_string(),
                    ));
                }
            }
            _ => {
                return Err(CommandError::InvalidArgument(
                    "Get command mast have a BulkString as the first argument".to_string(),
                ))
            }
        }
    }
    Ok(())
}

fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    // let mut args: Vec<&RespFrame> = Vec::with_capacity(value.len() - start);
    // for i in start..value.len() {
    //     args.push(&value[i]);
    // }
    // Ok(args)
    //// 链式
    Ok(value.0.into_iter().skip(start).collect::<Vec<RespFrame>>())
}
