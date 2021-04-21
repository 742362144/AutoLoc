use std::sync::{mpsc, Mutex, Arc};
use std::sync::mpsc::{Sender, Receiver};

use tonic::{transport::Server, Request, Response, Status};

use redis_module::{Context, RedisError, RedisResult, ThreadSafeContext, DetachedFromClient};

use funcloc::func_loc_server::{FuncLoc, FuncLocServer};
use funcloc::{InvokeRequest, InvokeReply, MultiRequest, MultiReply};
use std::thread;

use std::time::Duration;
use runtime::executor::Executor;
use runtime::invoke::Invoke;
use crate::policy::LocalPolicy;
use runtime::task::{Container, TaskMode};
use std::sync::atomic::{AtomicUsize, Ordering};

extern crate self_meter;

use std::io::{Write, stderr};
use std::thread::sleep;
use std::collections::{BTreeMap, VecDeque};
use rand::Rng;
use crate::THREADS;

pub mod funcloc {
    tonic::include_proto!("funcloc");
}

// #[derive(Default)]
pub struct MyGreeter {
    pub execs: VecDeque<Arc<Executor>>,
    pub index: Arc<AtomicUsize>,
}

impl MyGreeter {
    pub fn new() -> MyGreeter {
        MyGreeter {
            execs: VecDeque::new(),
            index: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[tonic::async_trait]
impl FuncLoc for MyGreeter {
    async fn invoke(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeReply>, Status> {
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let inv = Invoke { tx: Mutex::new(tx), req: String::from(request.into_inner().request), readSet: String::from("")};

        self.index.fetch_add(1, Ordering::SeqCst);
        let id = self.index.load(Ordering::SeqCst) as usize;
        let exec = self.execs.get(id % (THREADS as usize)).unwrap();

        let policy = exec.clone().policy.clone();
        let container = Container::new(id as u64, Box::new(inv), policy, TaskMode::LOCAL);
        exec.add_task(Box::new(container));

        // let res = rx.recv();
        // if !res.is_err() {
        //     println!("{}", res.unwrap());
        //     println!("Got a request from {:?}", request.remote_addr());
        // } else{
        //     println!("{}", "Error!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        // }


        // println!("Got a request from {:?}", request.remote_addr());
        let res = rx.recv().unwrap();
        if res == "success" {
            let reply = funcloc::InvokeReply {
                result: String::from("success"),
            };
            return Ok(Response::new(reply));
        }
        let reply = funcloc::InvokeReply {
            result: String::from(res),
        };
        return Ok(Response::new(reply));
    }

    async fn batch(
        &self,
        request: Request<MultiRequest>,
    ) -> Result<Response<MultiReply>, Status> {
        // println!("{}", "batching...");
        let invokes= request.into_inner().request;
        let mut que = Vec::new();
        let mut rxs = VecDeque::new();
        for r in invokes {
            let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
            // println!("{}", r.request);
            let inv = Invoke { tx: Mutex::new(tx), req: String::from(r.request), readSet: String::from("")};

            // let mut rng = rand::thread_rng();
            // let n = rng.gen_range(0, 100);
            self.index.fetch_add(1, Ordering::SeqCst);
            let id = self.index.load(Ordering::SeqCst) as usize;
            let exec = self.execs.get(id % (THREADS as usize)).unwrap();

            let policy = exec.clone().policy.clone();

            let container = Container::new(id as u64,Box::new(inv), policy, TaskMode::LOCAL);
            exec.add_task(Box::new(container));
            rxs.push_back(rx);
        }

        for rx in rxs {
            let res = rx.recv().unwrap();
            if res == "success" {
                let reply = funcloc::InvokeReply {
                    result: String::from("success"),
                };
                que.push(reply);
            } else {
                let reply = funcloc::InvokeReply {
                    result: res,
                };
                que.push(reply);
            }
        }

        let mut reply = funcloc::MultiReply {
            result: que,
        };
        return Ok(Response::new(reply));
    }
}

#[tokio::main]
pub async fn start_server(tctxs: VecDeque<ThreadSafeContext<DetachedFromClient>>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    let mut greeter = MyGreeter::new();
    let mut i:i32 = 0;
    let pushback = Arc::new(AtomicUsize::new(0));
    let counter = Arc::new(AtomicUsize::new(0));
    for tctx in tctxs {
        // println!("{}", String::from("start thread"));
        let policy = LocalPolicy::new(tctx);
        let exec = Arc::new(Executor::new(i.to_string(), counter.clone(), pushback.clone(), Arc::new(Mutex::new(policy))));
        let executor = exec.clone();
        greeter.execs.push_back(exec);
        thread::Builder::new().name("executor".to_string()).spawn(move || {
            executor.run();
        });
        i += 1;
    }

    // thread::spawn(move || {
    //     let mut meter = self_meter::Meter::new(Duration::new(1, 0)).unwrap();
    //     meter.track_current_thread("executor");
    //     loop {
    //         meter.scan()
    //             .map_err(|e| writeln!(&mut stderr(), "Scan error: {}", e)).ok();
    //         println!("Report: {:#?}", meter.report());
    //         println!("Threads: {:#?}",
    //                  meter.thread_report().map(|x| x.collect::<BTreeMap<_,_>>()));
    //         let mut x = 0;
    //         for _ in 0..10000000 {
    //             x = u64::wrapping_mul(x, 7);
    //         }
    //         sleep(Duration::new(1, 0));
    //     }
    // });


    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(FuncLocServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}