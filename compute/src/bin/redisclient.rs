use funcloc::func_loc_client::FuncLocClient;
use funcloc::{InvokeRequest, InvokeReply};
use std::sync::{mpsc, Mutex, Arc};
use std::time::Instant;
use runtime::executor::Executor;
use runtime::invoke::Invoke;
use std::sync::atomic::{AtomicUsize, Ordering};
use runtime::task::{Container, TaskMode};
use runtime::policy::Policy;
use compute::policy::{LocalPolicy, get_con};
use serde_json::json;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::env;
use std::collections::VecDeque;
use tokio::task::JoinHandle;
use std::io;
use std::io::prelude::*;
use std::fs::OpenOptions;
pub mod funcloc {
    tonic::include_proto!("funcloc");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //
    // let mut i:i32 = 0;
    // while i < n {
    //     let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    //     let policy = exec.clone().policy.clone();
    //     let inv = Invoke{tx: Mutex::new(tx), req: String::from("khop")};
    //     let container = Container::new(Box::new(inv), policy);
    //     exec.add_task(Box::new(container));
    //
    //     let res = rx.recv().unwrap();
    //     println!("{}", res);
    //     i += 1;
    // }

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
    let counter = Arc::new(AtomicUsize::new(0));
    let pushback = Arc::new(AtomicUsize::new(0));
    let policy = Mutex::new(policy);
    let mut name:i32 = 0;
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
                let f2 = fname.clone();
                let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
                let policy = e1.clone().policy.clone();
                let args = json!({
                        "f": f2,
                        "nodeId": j.to_string(),
                    });
                let inv = Invoke { tx: Mutex::new(tx), req: args.to_string(), readSet: String::from("")};
                let container = Container::new(0, Box::new(inv), policy, TaskMode::REMOTE);
                e1.add_task(Box::new(container));

                let res = rx.recv().unwrap();
                // println!("{}", res);

                // let reply = response.into_inner();

                // println!("RESPONSE={:?}", reply.result);
            });
            q.push_back(t1);
            j += 1;
        }

        while !q.is_empty() {
            let jh = q.pop_front();
            if let Some(mut jh) = jh {
                jh.await?;
            }
        }
        i = i + 1;
    }



    // t2.await?;
    let elapsed = start.elapsed();
    let filename1 = "/root/AutoControl/compute_exec.txt";
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

