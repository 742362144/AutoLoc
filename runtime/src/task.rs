// #![feature(in_band_lifetimes)]
#![feature(generators, generator_trait)]
#![feature(llvm_asm)]

use std::sync::mpsc::Sender;
use std::pin::Pin;
use std::ops::{Generator, GeneratorState};
use std::sync::{Mutex, Arc};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::thread;
use std::time::Duration;
use rand::Rng;
use std::error::Error;
use serde_json::{Value, Map};

use crate::invoke::Invoke;
use crate::cycles;
use crate::ext;
use crate::policy::Policy;


pub trait Task {
    /// When called, this method should "run" the task.
    ///
    /// # Return
    ///
    /// A tuple whose first member consists of the current state of the task
    /// (`TaskState`), and whose second member consists of the amount of time
    /// in cycles the task continuously ran for during this call to run().
    fn run(&mut self) -> (TaskState, u64);

    /// When called, this method should return the current state of the task.
    ///
    /// # Return
    ///
    /// The current state of the task (`TaskState`).
    fn state(&self) -> TaskState;

    fn is_local(&self) -> bool;

    /// When called, this method should return the total time for which the task
    /// has run since it was created.
    ///
    /// # Return
    ///
    /// The total time for which the task has run in cycles.
    fn time(&self) -> u64;

    /// When called, this method should return the total time for which the task
    /// has spent in db operations since it was created.
    ///
    /// # Return
    ///
    /// The total time for which the task has spent in db operations in cycles.
    fn db_time(&self) -> u64;

    fn count(&self) -> u64;

    fn overload(&mut self);

    fn readSet(&mut self, s: String);

    fn id(&self) -> u64;

    fn pushback(&self);

    fn finish(&self);
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum TaskState {
    /// A task is in this state when it has just been created, but has not
    /// had a chance to execute on the CPU yet.
    INITIALIZED = 0x01,

    /// A task is in this state when it is currently running on the CPU.
    RUNNING = 0x02,

    /// A task is in this state when it has got a chance to run on the CPU at
    /// least once, but has yeilded to the scheduler, and is currently not
    /// executing on the CPU.
    YIELDED = 0x03,

    /// A task is in this state when it has finished executing completely, and
    /// it's results are ready.
    COMPLETED = 0x04,

    /// A task is in this state when it has been stopped without completion, after
    /// setting this state, the pushback mechanism will run.
    STOPPED = 0x5,

    /// A task is in this state when it has been suspended due to IO. On the client side
    /// the task can wait for the native operation responses.
    WAITING = 0x6,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum TaskMode {
    /// A task is in this state when it has just been created, but has not
    /// had a chance to execute on the CPU yet.
    REMOTE = 0x01,

    /// A task is in this state when it is currently running on the CPU.
    LOCAL = 0x02,
}

pub struct Container {
    state: TaskState,
    pub index: u64,
    pub time: u64,
    pub db_time: u64,
    pub count: u64,
    inv: Box<Invoke>,
    gen: Option<Pin<Box<dyn Generator<Yield = u64, Return = u64>>>>,
    mode: TaskMode,
    readSet: String,
    policy: Arc<Mutex<Policy>>,
}

impl Container {
    pub fn new(cid: u64, inv: Box<Invoke>, policy: Arc<Mutex<Policy>>, mode: TaskMode) -> Container {
        // println!("{}", inv.req.as_str());
        let parsed: Value = serde_json::from_str(inv.req.as_str()).unwrap();
        let mut  obj= parsed.as_object().unwrap().clone();
        let readSet = inv.readSet.clone();
        obj.insert(String::from("readSet"), Value::String(readSet));
        let args = Arc::new(obj);

        let f = args.get("f").unwrap();

        let mut gen = None;
        if f == "khop" {
            gen = Some(ext::khop(args, policy.clone()));
        } else if f == "md5" {
            gen = Some(ext::md5(args, policy.clone()));
        } else if f == "pagerank" {
            gen = Some(ext::pagerank(args, policy.clone()));
        } else {
            gen = Some(ext::init(args, policy.clone()));
        }

        let mut con = Container {
            state: TaskState::INITIALIZED,
            index: cid,
            time: 0,
            db_time: 0,
            count: 0,
            inv,
            gen,
            mode,
            policy: policy.clone(),
            readSet: String::from(""),
        };
        // if req.as_str() == "khop" {
        //     con.gen = Some(ext::khop(obj, policy.clone()));
        // } else if req.as_str() == "md5" {
        //     con.gen = Some(ext::md5(obj, policy.clone()));
        // }

        con
    }

    // pub fn new(inv: Box<Invoke>, gen: Option<Pin<Box<dyn Generator<Yield = u64, Return = u64>>>>) -> Container {
    //     Container {
    //         state: TaskState::INITIALIZED,
    //         time: 0,
    //         inv,
    //         gen,
    //     }
    // }
}

impl Task for Container {
    fn run(&mut self) -> (TaskState, u64) {
        let start = cycles::rdtsc();

        // Resume the task if need be. The task needs to be run/resumed only
        // if it is in the INITIALIZED or YIELDED state. Nothing needs to be
        // done if it has already completed, or was aborted.
        if self.state == TaskState::INITIALIZED || self.state == TaskState::YIELDED {
            self.state = TaskState::RUNNING;

            // Catch any panics thrown from within the extension.
            let res = catch_unwind(AssertUnwindSafe(|| match self.gen.as_mut() {
                Some(gen) => match gen.as_mut().resume(()) {
                    GeneratorState::Yielded(db) => {
                        let end = cycles::rdtsc();
                        self.readSet = self.policy.lock().unwrap().get_readSet();
                        if db > 0 {
                            self.db_time += (1053011.58220886 + 100.17800923 * db as f64) as u64;
                        }
                        self.time += end - start;
                        // println!("{}", "yield...");
                        self.state = TaskState::YIELDED;
                    }

                    GeneratorState::Complete(_) => {
                        // println!("time: {}", self.time);
                        // println!("db_timee: {}", self.db_time);
                        self.state = TaskState::COMPLETED;
                        // println!("{}", "complete...");
                    }
                },

                None => {
                    panic!("No generator available for extension execution");
                }
            }));

            // If there was a panic thrown, then mark the container as COMPLETED so that it
            // does not get run again.
            // if let Err(_) = res {
            //     self.state = TaskState::COMPLETED;
            //     if thread::panicking() {
            //         // Wait for 100 millisecond so that the thread is moved to the GHETTO core.
            //         let start = cycles::rdtsc();
            //         while cycles::rdtsc() - start < cycles::cycles_per_second() / 10 {}
            //     }
            // }
        }

        // Calculate the amount of time the task executed for in cycles.
        let exec = cycles::rdtsc() - start;

        // Update the total execution time of the task.
        self.time += exec;

        // Return the state and the amount of time the task executed for.
        return (self.state, exec);
    }

    fn state(&self) -> TaskState {
        unimplemented!()
    }

    fn time(&self) -> u64 {
        self.time
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn overload(&mut self) {
        self.count += 1;
    }

    fn id(&self) -> u64 {
        self.index
    }
    fn db_time(&self) -> u64 {
        self.db_time
    }

    fn is_local(&self) -> bool {
        if self.mode == TaskMode::LOCAL {
            true
        } else {
            false
        }
    }

    fn readSet(&mut self, s: String) {
        self.readSet = s;
    }

    fn pushback(&self) {
        let s = self.readSet.clone();
        self.inv.tx.lock().unwrap().send(s);
    }

    fn finish(&self) {
        // let mut rng =rand::thread_rng();
        // let n = rng.gen_range(0, 100);
        // if n % 3 == 0 {
        //     self.inv.tx.lock().unwrap().send(String::from("pushback"));
        // } else {
        //     self.inv.tx.lock().unwrap().send(String::from("success"));
        // }
        self.inv.tx.lock().unwrap().send(String::from("success")).unwrap();
    }
}

