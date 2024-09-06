use crate::{RespArray, RespFrame};
use crate::cmd::{extract_args, validate_command, CommandError, Get, HGet, HGetAll, HSet, Set};

impl TryFrom<RespArray> for HGet{
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value,&["hget"],2)?;
        let mut args = extract_args(value,1)?.into_iter();
        match (args.next(),args.next()){
            (Some(RespFrame::BulkString(key)),Some(RespFrame::BulkString(field)))=>Ok(HGet{
                key:String::from_utf8(key.0)?,
                field:String::from_utf8(field.0)?
            }),
            _=>Err(CommandError::InvalidArgument("Invalid key field".to_string()))
        }
    }
}

impl TryFrom<RespArray> for HGetAll{
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value,&["hgetall"],1)?;
        let mut args = extract_args(value,1)?.into_iter();
        match args.next(){
            Some(RespFrame::BulkString(key))=>Ok(HGetAll{
                key:String::from_utf8(key.0)?
            }),
            _=>Err(CommandError::InvalidArgument("Invalid key ".to_string()))
        }
    }
}
impl TryFrom<RespArray> for HSet{
    type Error = CommandError;
}