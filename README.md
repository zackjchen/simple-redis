# Simple-redis


一个简单的redis server的实现

## Bytes
摘抄自 <a href="https://tokio.rs/tokio/tutorial/shared-state" >https://tokio.rs/tokio/tutorial/shared-state</a>

**Bytes就是`Vec<u8>`**

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

## futures
```sh
# 添加lib时不带默认的features
cargo add futures --no-default-features
```
**`Stream`类似`Iterator`，是一个异步读取的trait，需要实现`poll_next`方法。**

**`Sink`是一个异步写的trait，至少需要实现`poll_ready`方法**

- `StreamExt`和`SinkExt`都是工具trait，在`Stream`和`Sink`的基础上提供一些封装的方法
  标准库提供了Future trait用于支持异步。

- futures是第三方对异步编程的支持和抽象，提供Sink和Stream, 以及SinKExt, StreamExt
  tokio只提供了Stream trait，并且和futures提供的Stream有所不同.

- tokio-util提供compat功能可以转换两个Stream


<img src='https://raw.githubusercontent.com/zackjchen/simple-redis/master/asserts/%E5%BC%82%E6%AD%A5io%E6%8E%A5%E5%8F%A3.webp'>

```rust
// StreamExt, iterator有的它也有
fn next
fn map
....
// SinkExt
fn send
fn send_all
```

## enum_dispatch

```rust
use enum_dispatch::enum_dispatch;

struct A;
struct B;
struct C;

#[enum_dispatch]
trait MyTrait {
    fn execute(self);
}
impl MyTrait for A{...}
impl MyTrait for B{...}
impl MyTrait for C{...}

[dispatch(MyTrait)]
enum E{
    FieldA(A)
    FieldB(B)
    FieldC(C)
    ...
}
/// 上面的写法自动实现下面的代码
///  -----------------------------------
impl From<A> for E{
    fn from(set: Set) -> Self {
        Command::A(A)
    }
}
impl From<B> for E{
    fn from(set: Set) -> Self {
        Command::B(B)
    }
}
impl From<C> for E{
    fn from(set: Set) -> Self {
        Command::C(C)
    }
}
impl MyTrait for E {
    fn execute(self){
        match E{
            A => A.execute(),
            B => B.execute(),
            C => C.execute(),
        }
    }
}
```
