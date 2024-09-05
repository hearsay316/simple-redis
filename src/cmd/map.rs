use crate::{RespArray, RespFrame};
use crate::cmd::{extract_args, validate_command, CommandError, Get, Set};


impl TryFrom<RespArray> for Get{
        type Error = CommandError;
    fn try_from(value:RespArray)->Result<Self,Self::Error>{
        validate_command(&value,&["get"],1)?;
        let  args = extract_args(&value,1)?;
        match args[0] {
            RespFrame::BulkString(key) => Ok(Get { key: String::from_utf8_lossy(key).to_string() })
            ,
            _ => Err(CommandError::InvalidArgument("Invalid key ".to_string()))
        }
    }
}
impl TryFrom<RespArray> for Set{
    type Error= CommandError;
    fn try_from(value:RespArray)->Result<Self,Self::Error>{

    }
}
#[cfg(test)]
mod tests{
    use bytes::BytesMut;
    use crate::RespDecode;
    use anyhow::Result;
    use super::*;
    #[test]
    fn test_get_try_resp_array()->Result<()>{
        let mut buf = BytesMut::new();

        buf.extend_from_slice( b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;

        let result:Get = frame.try_into()?;

        assert_eq!(result.key,"hello");
        Ok(())
    }
}