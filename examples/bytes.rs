use anyhow::Result;
use bytes::{BufMut, BytesMut};

fn main() -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);
    buf.extend_from_slice(b"hello world\n");
    buf.put(&b"goodbye world"[..]);
    buf.put_i64(0xdeadbeef);

    println!("{:?}", buf); // b"hello world\ngoodbye world\0\0\0\0\xde\xad\xbe\xef"

    // buf 里面就没东西了，a 拥有 buf 里的所有数据，为 BytesMut 类型
    let a = buf.split();
    println!("{:?}", a); // b"hello world\ngoodbye world\0\0\0\0\xde\xad\xbe\xef"
    println!("{:?}", buf); // b""
    println!("-------------------");

    // a 就不能继续访问了，因为传进来的是 self 所有权，b 拥有 a 的所有权，类型变成了 Bytes
    let mut b = a.freeze();

    // 返回 [0, 12) 的数据，剩下的是 [12, end]
    let c = b.split_to(12);
    println!("{:?}", c); // b"hello world\n"

    println!("{:?}", b); // b"goodbye world\0\0\0\0\xde\xad\xbe\xef"
    println!("{:?}", buf); // b""

    Ok(())
}
