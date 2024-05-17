#![allow(dead_code)]
use crate::{
    backend::Backend,
    resp::{RespArray, RespFrame, SimpleString},
};
use lazy_static::lazy_static;
use thiserror::Error;
mod hmap;
mod map;
lazy_static! {
    static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}
pub trait CommandExecuter {
    fn execute(self, backend: Backend) -> RespFrame;
}

pub enum Command {
    Get(Get),
    Set(Set),
    Del(Del),
    HSet(HSet),
    HGet(HGet),
    HGetAll(HGetAll),
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

#[derive(Debug)]
pub struct Del {
    key: String,
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

    fn try_from(_frame: RespArray) -> Result<Self, Self::Error> {
        todo!()
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
