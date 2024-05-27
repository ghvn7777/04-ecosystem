use std::{thread, time::Duration};

use tokio::{
    fs,
    runtime::{Builder, Runtime},
    time::sleep,
};

fn main() {
    let handle = thread::spawn(|| {
        let rt = Builder::new_multi_thread().enable_all().build().unwrap();
        rt.block_on(run(&rt));
    });

    handle.join().unwrap();
}

async fn run(rt: &Runtime) {
    rt.spawn(async {
        println!("Future 1");
        let content = fs::read("Cargo.toml").await.unwrap();
        println!("content: {:?}", content.len());
    });

    rt.spawn(async {
        println!("Future 2");
        let ret = expensive_blocking_task("hello".to_string());
        println!("ret: {:?}", ret);
    });

    sleep(Duration::from_secs(1)).await;
}

fn expensive_blocking_task(s: String) -> String {
    thread::sleep(Duration::from_millis(700));
    blake3::hash(s.as_bytes()).to_string()
}
