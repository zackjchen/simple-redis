# Geektime Rust 语言训练营 Simple-redis


一个简单的redis server的实现

## Bytes
摘抄自 `https://tokio.rs/tokio/tutorial/shared-state`
Bytes就是Vec<u8>
Instead of using Vec<u8>, the Mini-Redis crate uses Bytes from the bytes crate.  The goal of Bytes is to provide a robust byte array structure for network programming.  The biggest feature it adds over Vec<u8> is shallow cloning.  In other words, calling clone() on a Bytes instance does not copy the underlying data.  Instead, a Bytes instance is a reference-counted handle to some underlying data.  The Bytes type is roughly an Arc<Vec<u8>> but with some added capabilities.

**BytesMut中重要的3个方法**
+ `split`: 将一个BytesMut分成两个，self: len到结束，返回值0-len，等于`split_to(self.len)`
+ `sunsplit`: 将两个BytesMut合并成一个
+ `freeze`: 将BytesMut转换为Bytes


```rust
use bytes::{BytesMut, BufMut};

let mut buf = BytesMut::with_capacity(1024);
buf.put(&b"hello world"[..]);
buf.put_u16(1234);

// split
let a = buf.split();
assert_eq!(a, b"hello world\x04\xD2"[..]);

buf.put(&b"goodbye world"[..]);

let b = buf.split();
assert_eq!(b, b"goodbye world"[..]);

assert_eq!(buf.capacity(), 998);

// unsplit
let mut buf = BytesMut::with_capacity(64);
buf.extend_from_slice(b"aaabbbcccddd");

let split = buf.split_off(6);
assert_eq!(b"aaabbb", &buf[..]);
assert_eq!(b"cccddd", &split[..]);

buf.unsplit(split);
assert_eq!(b"aaabbbcccddd", &buf[..]);
```


## test
```shell
cargo nextest run
cargo nextest run fn_test_name
```


## 好的写法
```rust
/// 这个写法需要好好记一下
impl <const N: usize> From<&[u8;N]> for BulkString {
    fn from(s: &[u8;N]) -> Self {
        BulkString(s.to_vec())
    }
}
```

## DashMap
```rust
let map = DashMap::new();
/// 下面三个方法获取的返回值都可以获取key, 和value
map.entry(key) // 返回的是一个entry，可读可写,可插入，可修改
map.get_mut(key) // 返回的是一个可变引用，但不能插入值，只能修改
map.get(key)   // 返回的是一个引用，只可读
```
