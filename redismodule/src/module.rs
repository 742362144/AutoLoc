use redis_module::{redis_module, redis_command};
use redis_module::{Context, RedisError, RedisResult, ThreadSafeContext};
use std::thread;
use log::{error, info};
use std::error::Error;
use std::path::Path;
use std::{fmt, fs};

use std::time::Duration;
use crate::server::start_server;
use std::collections::VecDeque;
use crate::THREADS;


fn service(_: &Context, _args: Vec<String>) -> RedisResult {
    thread::spawn(move || {
        let mut que = VecDeque::new();
        let mut i = 0;
        while i < THREADS {
            let thread_ctx = ThreadSafeContext::new();
            que.push_back(thread_ctx);
            i += 1;
        }
        start_server(que);
        // print_world();

        // for _ in 0..2 {
        //     let ctx = thread_ctx.lock();
        //     ctx.call("INCR", &["threads"]).unwrap();
        //     thread::sleep(Duration::from_millis(100));
        // }
    });

    Ok(().into())
}

//////////////////////////////////////////////////////
redis_module! {
    name: "service",
    version: 1,
    data_types: [],
    commands: [
        ["service", service, "", 0, 0, 0],
    ],
}