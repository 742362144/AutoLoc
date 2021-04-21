use std::sync::{Arc, Mutex, RwLock};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Sender, Receiver};
use rand::Rng;

use crate::task::{Task, TaskState};
use crate::policy::Policy;
use std::thread;
use std::time::Duration;
use crate::{cycles, PARALLEL, LIMIT};
use std::time::Instant;
use crate::cycles::rdtsc;
// use std::io;
// use std::io::prelude::*;
// use std::fs::OpenOptions;

pub struct Executor {
    pub name: String,
    pub counter: Arc<AtomicUsize>,
    pub pushback: Arc<AtomicUsize>,
    pub policy: Arc<Mutex<Policy>>,
    pub waiting: RwLock<VecDeque<Box<Task>>>,
    pub running: RwLock<VecDeque<Box<Task>>>,
}

impl Executor {
    pub fn new(name: String, counter: Arc<AtomicUsize>, pushback: Arc<AtomicUsize>, policy: Arc<Mutex<Policy>>) -> Executor {
        Executor {
            name,
            pushback,
            counter,
            policy,
            waiting: RwLock::new(VecDeque::new()),
            running: RwLock::new(VecDeque::new()),
        }
    }

    pub fn add_task(&self, task: Box<Task>) {
        self.waiting.write().unwrap().push_back(task);
    }

    pub fn run(&self) {
        let mut busy_time = Instant::now();
        let mut pushback = 0;
        let mut cycle = 0;

        // let filename1 = "/tmp/counter.txt";
        // let mut file1 = OpenOptions::new()
        //     .read(true)
        //     .write(true)
        //     .create(true)
        //     //.create_new(true)
        //     .append(true)
        //     .open(filename1).unwrap();

        // file1.write_all(self.counter.load(Ordering::SeqCst).to_string().as_bytes()).unwrap();
        //
        // let filename2 = "/tmp/pushback.txt";
        // let mut file2 = OpenOptions::new()
        //     .read(true)
        //     .write(true)
        //     .create(true)
        //     //.create_new(true)
        //     .append(true)
        //     .open(filename2).unwrap();
        //
        // file2.write_all(self.pushback.load(Ordering::SeqCst).to_string().as_bytes()).unwrap();
        loop {
            if self.waiting.write().unwrap().len() == 0 && self.running.write().unwrap().len() == 0 {
                let elapsed = busy_time.elapsed();
                // println!("{}", "sleeping...");
                if elapsed.as_secs() > 20 {
                    thread::sleep(Duration::from_micros(1));
                }
            }

            let w_len = self.waiting.read().unwrap().len() as i32;
            let r_len = self.running.read().unwrap().len() as i32;

            // println!("w_len: {}", w_len);
            // println!("r_len: {}", r_len);
            if w_len > 0 && r_len < PARALLEL {
                let mut i = 0;
                while i < PARALLEL - r_len {
                    let task = self.waiting.write().unwrap().pop_front();
                    if let Some(mut task) = task {
                        self.running.write().unwrap().push_back(task);
                    }
                    i = i + 1;
                }
            }

            let mut r_len = self.running.read().unwrap().len() as i32;
            while r_len > 0 {
                let task = self.running.write().unwrap().pop_front();
                if let Some(mut task) = task {
                    if task.run().0 == TaskState::COMPLETED {
                        // println!("{}", "task finish.");
                        task.finish();
                        self.counter.fetch_add(1, Ordering::SeqCst);
                        let counter = self.counter.load(Ordering::SeqCst);
                        println!("finish: {0}, id: {1}, time: {2}, db_time: {3}", counter, task.id(), task.time(), task.db_time());
                        // } else if task.time() > (task.db_time() as f64 * 2.0) as u64 && task.is_local() {
                        //     task.pushback();
                    } else {
                        // println!("{}", "check server overhead.");
                        self.running.write().unwrap().push_back(task);
                    }
                }

                busy_time = Instant::now();
                r_len -= 1;
            }

            let mut r_len = self.running.read().unwrap().len() as i32;
            while r_len > 0 {
                let task = self.running.write().unwrap().pop_front();
                if let Some(mut task) = task {
                    if task.time() > task.db_time() {
                        task.overload();
                        if task.is_local() && task.count() > LIMIT {
                            pushback += 1;
                            self.pushback.fetch_add(1, Ordering::SeqCst);
                            task.pushback();
                            let pushback = self.pushback.load(Ordering::SeqCst);
                            println!("pushback: {0}, id: {1}, time: {2}, db_time: {3}", pushback, task.id(), task.time(), task.db_time());
                        } else {
                            self.running.write().unwrap().push_back(task);
                        }
                    } else {
                        // println!("{}", "check server overhead.");
                        self.running.write().unwrap().push_back(task);
                    }
                }
                r_len -= 1;
                busy_time = Instant::now();
            }


            // let len = self.waiting.read().unwrap().len();
            // if len >= (PARALLEL / 2) as usize {
            //     // println!("{}", "overload..., start pushback...");
            //
            //     let mut  r_len = self.running.read().unwrap().len() as i32;
            //     while r_len > 0 {
            //         let task = self.running.write().unwrap().pop_front();
            //         if let Some(mut task) = task {
            //             if task.time() > (task.db_time() as f64 * 2.0) as u64 {
            //                 if task.is_local() {
            //                     pushback += 1;
            //                     task.pushback();
            //                     println!("overload..., pushback: {}", pushback);
            //                 } else {
            //                     self.running.write().unwrap().push_back(task);
            //                     break;
            //                 }
            //             } else {
            //                 // println!("{}", "check server overhead.");
            //                 self.running.write().unwrap().push_back(task);
            //             }
            //         }
            //         r_len -= 1;
            //     }
            //
            //     // let mut i = 0;
            //     // while i < (PARALLEL / 4) {
            //     //     let task = self.waiting.write().unwrap().pop_front();
            //     //     if let Some(mut task) = task {
            //     //         if task.is_local() {
            //     //             task.pushback();
            //     //         } else {
            //     //             self.waiting.write().unwrap().push_back(task);
            //     //         }
            //     //     }
            //     //     i += 1;
            //     // }
            // }
        }
    }
}

unsafe impl Send for Executor {}

unsafe impl Sync for Executor {}
