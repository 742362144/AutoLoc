use funcloc::func_loc_client::FuncLocClient;
use funcloc::{InvokeRequest, InvokeReply};
use std::sync::{mpsc, Mutex, Arc};
use std::time::Instant;
use runtime::executor::Executor;
use runtime::invoke::Invoke;
use runtime::task::{Container, TaskMode};
use runtime::policy::Policy;
use compute::policy::{LocalPolicy, get_con};
use serde_json::json;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::env;
use std::collections::VecDeque;
use tokio::task::JoinHandle;

pub mod funcloc {
    tonic::include_proto!("funcloc");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut num: i32 = 1;
    let s1 = num.to_string();
    println!("{}", s1.len());

    let s2 = String::from("1");
    println!("{}", s2.len());

    // let con = get_con();
    // let mut policy = LocalPolicy::new(con);
    // let mut i= 0;
    // while i < 1000 {
    //     let s = policy.get(i.to_string().as_str());
    //     // println!("{}", s);
    //     i += 1;
    // }
    Ok(())
}

