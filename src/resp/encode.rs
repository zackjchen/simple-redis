use super::{
    BulkString, RespArray, RespEncode, RespMap, RespNull, RespNullArray, RespNullBulkString,
    RespSet, SimpleError, SimpleString,
};
const BUF_CAPACITY: usize = 4096;

/*

RESP data type	        Minimal protocol version	Category	      First byte
Simple strings	        RESP2	                    Simple	          +
Simple Errors	        RESP2	                    Simple	          -
Integers	            RESP2	                    Simple	          :
Bulk strings	        RESP2	                    Aggregate	      $
Arrays	                RESP2	                    Aggregate	      *
Nulls	                RESP3	                    Simple	          _
Booleans	            RESP3	                    Simple	          #
Doubles	                RESP3	                    Simple	          ,
Maps	                RESP3	                    Aggregate	      %
Sets	                RESP3	                    Aggregate	      ~


*/

// impl RespEncode for RespFrame {
//     fn encode(self) -> Vec<u8> {
//         todo!()
//     }

// }

impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", *self).into_bytes()
    }
}

impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", *self).into_bytes()
    }
}

impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        // 如果是负数，format自己会加上负号，正数会省略所以补上
        let sign = if self < 0 { "" } else { "+" };
        format!(":{}{}\r\n", sign, self).into_bytes()
    }
}

impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(format!("${}\r\n", self.len()).as_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

impl RespEncode for RespNullBulkString {
    fn encode(self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}
impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAPACITY);
        buf.extend_from_slice(&format!("*{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}
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

impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { 't' } else { 'f' }).into_bytes()
    }
}

impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        format!(",{:+e}\r\n", self).into_bytes()
    }
}

impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAPACITY);
        buf.extend_from_slice(&format!("%{}\r\n", self.0.len()).into_bytes());
        for (k, v) in self.0 {
            // buf.extend_from_slice(format!("+{}\r\n", k).as_bytes());
            buf.extend_from_slice(&SimpleString::new(k).encode());
            buf.extend_from_slice(&v.encode());
        }
        buf
    }
}

impl RespEncode for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAPACITY);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use crate::resp::{
        BulkString, RespArray, RespEncode, RespFrame, RespMap, RespNullArray, RespNullBulkString,
        SimpleError, SimpleString,
    };

    #[test]
    fn test_simple_string_encode() {
        let frame: RespFrame = SimpleString::new("OK").into();
        assert_eq!(frame.encode(), b"+OK\r\n");
    }

    #[test]
    fn test_simple_error_encode() {
        let frame: RespFrame = SimpleError::new("Error message").into();
        assert_eq!(frame.encode(), b"-Error message\r\n");
    }

    #[test]
    fn test_integer_encode() {
        let frame: RespFrame = 123.into();
        assert_eq!(frame.encode(), b":+123\r\n");

        let frame: RespFrame = (-123).into();
        assert_eq!(frame.encode(), b":-123\r\n");
    }

    #[test]
    fn test_bulk_string_encode() {
        let frame: RespFrame = BulkString::new(b"foobar".to_vec()).into();
        assert_eq!(frame.encode(), b"$6\r\nfoobar\r\n");
    }

    #[test]
    fn test_null_bulk_string_encode() {
        let frame: RespFrame = RespNullBulkString.into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_array_encode() {
        let array: Vec<RespFrame> = vec![
            SimpleString::new("foo").into(),
            SimpleString::new("bar").into(),
            BulkString::new(b"baz".to_vec()).into(),
        ];
        let frame: RespFrame = RespArray(array).into();
        assert_eq!(
            String::from_utf8_lossy(&frame.encode()),
            "*3\r\n+foo\r\n+bar\r\n$3\r\nbaz\r\n"
        );
    }

    #[test]
    fn test_null_array() {
        let frame: RespFrame = RespNullArray.into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_boolean_encode() {
        let frame: RespFrame = true.into();
        assert_eq!(frame.encode(), b"#t\r\n");

        let frame: RespFrame = false.into();
        assert_eq!(frame.encode(), b"#f\r\n");
    }

    #[test]
    fn test_double_encode() {
        let frame: RespFrame = 5.21.into();
        assert_eq!(String::from_utf8_lossy(&frame.encode()), ",+5.21e0\r\n");

        let frame = RespFrame::from(-5.21);
        assert_eq!(frame.encode(), b",-5.21e0\r\n");

        let frame: RespFrame = 1.23456e+8.into();
        assert_eq!(String::from_utf8_lossy(&frame.encode()), ",+1.23456e8\r\n");

        let frame: RespFrame = (-1.23456e-8).into();
        assert_eq!(frame.encode(), b",-1.23456e-8\r\n");
    }

    #[test]
    fn test_map_encode() {
        let mut map = RespMap::new();
        map.insert("hello".into(), SimpleString::new("world").into());
        map.insert("foo".into(), (-123456.789).into());
        let frame: RespFrame = map.into();
        assert_eq!(
            String::from_utf8_lossy(&frame.encode()),
            "%2\r\n+foo\r\n,-1.23456789e5\r\n+hello\r\n+world\r\n"
        );
    }

    #[test]
    fn test_set_encode() {
        let set = vec![
            SimpleString::new("foo").into(),
            SimpleString::new("bar").into(),
            RespFrame::Double(-5.21),
            RespArray(vec![SimpleString::new("hello").into()]).into(),
        ];
        let frame: RespFrame = RespArray(set).into();
        assert_eq!(
            frame.encode(),
            b"*4\r\n+foo\r\n+bar\r\n,-5.21e0\r\n*1\r\n+hello\r\n"
        );
    }
}
