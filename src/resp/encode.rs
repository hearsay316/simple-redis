use crate::{BulkString, RespArray, RespEncode, RespFrame, RespMap, RespNull, RespNullArray, RespSet, SimpleError, SimpleString};

/*
- 如何解析 Frame
    - simple string: "+OK\r\n"
    - error: "-Error message\r\n"
    - bulk error: "!<length>\r\n<error>\r\n"
    - integer: ":[<+|->]<value>\r\n"
    - bulk string: "$<length>\r\n<data>\r\n"
    - null bulk string: "$-1\r\n"
    - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
        - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
    - null array: "*-1\r\n"
    - null: "_\r\n"
    - boolean: "#<t|f>\r\n"
    - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
    - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
    - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
 */
const BUF_CAP: usize = 4096;

// impl RespEncode for RespFrame {
//     fn encode(self) -> Vec<u8> {
//         todo!()
//     }
// }

//- simple string:"+OK\r\n"
impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

// - bulk string: "$<length>\r\n<data>\r\n"
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        // Vec::with_capacity()方法创建一个具有指定容量的Vec<u8>，这样可以避免在push()时重新分配内存
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
        // format!("${}\r\n{}\r\n",self.len(),String::from_utf8(self).unwrap()).into_bytes()
        // format!("${}\r\n{}\r\n",self.len(),String::from_utf8_lossy(&self)).into_bytes()
    }
}

// - null bulk string: "$-1\r\n"
impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

impl RespEncode for RespNullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

//error: "-Error message\r\n"
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

//- integer: ":[<+|->]<value>\r\n"
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        let sign = if self < 0 { "" } else { "+" };
        format!(":{}{}\r\n", sign, self).into_bytes()
    }
}

// array: "*<number-of-elements>\r\n<element-1>...<element-n>"


impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

//    - boolean: "#<t|f>\r\n"
impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { "t" } else { "f" }).into_bytes()
    }
}

//  - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        let ret = if self.abs() > 1e+8 || self.abs() < 1e-8 {
            format!(",{:+e}\r\n", self)
        } else {
            let sign = if self < 0.0 { "" } else { "+" };
            format!(",{}{}\r\n", sign, self)
        };
        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

//map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        for (key, value) in self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}
// "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespEncode for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode())
        }
        buf
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simple_string_encode() {
        let frame:RespFrame = SimpleString::new("OK".to_string()).into();
        assert_eq!(frame.encode(), b"+OK\r\n");
    }
    #[test]
    fn test_simple_error_encode(){
        let frame:RespFrame = SimpleError::new("Error message".to_string()).into();
        assert_eq!(frame.encode(),b"-Error message\r\n");
    }
    #[test]
    fn test_integer_encode(){
        let frame:RespFrame = 123.into();
        assert_eq!(frame.encode(),b":+123\r\n");
        let frame:RespFrame = (-123).into();
        assert_eq!(frame.encode(),b":-123\r\n");
    }
    #[test]
    fn test_bulk_string_encode(){
        let frame:RespFrame = BulkString::new("OK".to_string()).into();
        assert_eq!(frame.encode(),b"$2\r\nOK\r\n");
    }
    #[test]
    fn test_array_encode(){
        let frame:RespFrame = RespArray::new(vec![
            BulkString::new("set".to_string()).into(),
            BulkString::new("hello".to_string()).into(),
            BulkString::new("world".to_string()).into(),
        ]).into();
        // println!("{:?}",frame.encode().);
        // assert_eq!(frame.encode(),b"*3\r\n+set\r\n+hello\r\n+world\r\n");
        assert_eq!(frame.encode(),b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
    }
    #[test]
    fn test_null_encode(){
        let frame:RespFrame = RespNull::new().into();

        assert_eq!(frame.encode(),b"_\r\n");
    }
    #[test]
    fn test_boolean_encode(){
        let frame:RespFrame = true.into();
        assert_eq!(frame.encode(),b"#t\r\n");
        let frame:RespFrame = false.into();
        assert_eq!(frame.encode(),b"#f\r\n");
    }
    #[test]
    fn test_double_encode(){
        let frame:RespFrame = 123.456.into();
        assert_eq!(frame.encode(),b",+123.456\r\n");
        let frame:RespFrame = (-123.456).into();
        assert_eq!(frame.encode(),b",-123.456\r\n");
        let frame:RespFrame = 1.23456e+8.into();
        assert_eq!(frame.encode(),b",+1.23456e8\r\n");
        let frame:RespFrame = (-1.23456e-9).into();
        assert_eq!(frame.encode(),b",-1.23456e-9\r\n");
    }
    #[test]
    fn test_map_encode(){
        let mut map: RespMap = RespMap::new();
        map.insert("key".to_string(),SimpleString::new("value".to_string()).into());
        let frame:RespFrame = map.into();
       assert_eq!(frame.encode(),b"%1\r\n+key\r\n+value\r\n");
    }
    #[test]
    fn test_set_encode(){
        let  frame: RespSet = RespSet::new([
            SimpleString::new("value".to_string()).into(),
            BulkString::new("world".to_string()).into()
        ]).into();
       assert_eq!(frame.encode(),b"~2\r\n+value\r\n$5\r\nworld\r\n");
    }
}
