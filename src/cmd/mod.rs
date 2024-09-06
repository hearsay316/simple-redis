mod map;
mod hmap;

use thiserror::Error;
use crate::{RespArray, RespError, RespFrame};
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command:{0}")]
    InvalidCommand(String),
    #[error("Invalid number of arguments:{0} ")]
    InvalidNumberOfArguments(usize),
    #[error("Invalid argument:{0}")]
    InvalidArgument(String),
    #[error("{0}")]
    RespError(#[from] RespError),

    #[error("Utf8 error :{0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
pub trait CommandExecutor {
    fn execute(self) -> RespFrame;
}
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HGetALl(HGetAll),
    // Del,
    // Incr,
    // Decr,
    // Exists,
    // Keys,
    // Ping,
    // Quit,
    // Unknown,
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
pub struct HGet {
    key: String,
    field: String,
}
#[derive(Debug)]

pub struct HSet {
    ket: String,
    field: String,
    value: RespFrame,
}
#[derive(Debug)]
pub struct HGetAll {
    key: String,
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        todo!()
    }

}
// fn validate_command(value:&RespArray,names:&[&'static str],n_args:usize)->Result<(),CommandError>{
//     if value.len() != n_args+names.len(){
//         return Err(CommandError::InvalidArgument(format!("{} command must have exactly {} argument",names.join(" "),n_args)))
//     };
//     for (index, name) in names.iter().enumerate() {
//         match value[index] {
//             RespFrame::BulkString(ref cmd) => {
//                 if cmd.as_ref().to_ascii_lowercase() != name.as_bytes() {
//                     return Err(CommandError::InvalidCommand(format!("Invalid  command: expected {}, got {}", name, String::from_utf8_lossy(cmd.as_ref()))))
//                 }
//             },
//             _ => {
//                 return Err(CommandError::InvalidArgument("command must have exactly {} argument"))
//             }
//         }
//     }
//     Ok(())
// }

fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have exactly {} argument",
            names.join(" "),
            n_args
        )));
    }

    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.as_ref().to_ascii_lowercase() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: expected {}, got {}",
                        name,
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "Command must have a BulkString as the first argument".to_string(),
                ))
            }
        }
    }
    Ok(())
}
fn extract_args(value:RespArray,start:usize)->Result<Vec<RespFrame>,CommandError>{
    Ok(value.0.into_iter().skip(start).collect::<Vec<RespFrame>>())
}