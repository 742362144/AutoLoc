use funcloc::func_loc_client::FuncLocClient;
use funcloc::{InvokeRequest, InvokeReply};
use std::time::Instant;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Mutex, Arc};
use runtime::executor::Executor;
use runtime::invoke::Invoke;
use runtime::task::{Container, TaskMode};
use runtime::policy::Policy;
use compute::policy::{LocalPolicy, get_con};

use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use serde_json::json;
use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::OpenOptions;
extern crate clap;

use clap::{Arg, App, SubCommand, ArgMatches};
use std::collections::VecDeque;
use tokio::task::JoinHandle;

pub mod funcloc {
    tonic::include_proto!("funcloc");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = env::args().collect();
    // for arg in args {
    //     println!("{:?}", arg);
    // }
    let n = args.pop().unwrap().parse::<i32>().unwrap();
    let t = args.pop().unwrap().parse::<i32>().unwrap();
    let f = args.pop().unwrap();

    // let f = "khop";
    // let t = 1;
    // let n = 10;

    let con = get_con();
    let mut policy = LocalPolicy::new(con);

    let policy = Mutex::new(policy);
    let mut name = 0;
    let counter = Arc::new(AtomicUsize::new(0));
    let pushback = Arc::new(AtomicUsize::new(0));
    let exec = Arc::new(Executor::new(name.to_string(), counter, pushback, Arc::new(policy)));
    let executor = exec.clone();

    thread::spawn(move || {
        executor.run();
    });

    let start = Instant::now();

    let mut q = VecDeque::new();
    let mut i = 0;
    while i < t {
        let mut j: i32 = 1;
        while j <= n {
            let e1 = exec.clone();
            let fname = f.clone();
            let t1 = tokio::spawn(async move {
                let mut client = FuncLocClient::connect("http://[::1]:50051").await.unwrap();
                let f1 = fname.clone();
                let f2 = fname.clone();

                let args = json!({
                        "f": f1,
                        "nodeId": j.to_string(),
                    });
                let request = tonic::Request::new(InvokeRequest {
                    request: args.to_string().into(),
                });
                let response = client.invoke(request).await;
                let reply = response.unwrap().into_inner();
                if reply.result == "success" {
                    // println!("RESPONSE={:?}", reply.result);
                } else {
                    // println!("pushback={:?}", reply.result);
                    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
                    let policy = e1.clone().policy.clone();
                    let args = json!({
                        "f": f2,
                        "nodeId": j.to_string(),
                    });
                    let inv = Invoke { tx: Mutex::new(tx), req: args.to_string(), readSet: reply.result};
                    let container = Container::new(0, Box::new(inv), policy, TaskMode::REMOTE);
                    e1.add_task(Box::new(container));

                    let res = rx.recv().unwrap();
                    // println!("{}", res);
                }
            });

            q.push_back(t1);
            // let reply = response.into_inner();

            // println!("RESPONSE={:?}", reply.result);
            j += 1;
        }
        while !q.is_empty() {
            let jh = q.pop_front();
            if let Some(mut jh) = jh {
                jh.await.unwrap();
            }
        }
        i = i + 1;
    }



    // t2.await?;
    let elapsed = start.elapsed();
    let filename1 = "/root/AutoControl/storageloc_exec.txt";
    let mut file1 = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        //.create_new(true)
        .append(true)
        .open(filename1).unwrap();
    file1.set_len(0);
    file1.write_all(elapsed.as_secs_f64().to_string().as_bytes()).unwrap();
    // Debug format
    println!("Debug: {:?}", elapsed);
    Ok(())
}

