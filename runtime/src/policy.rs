use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::time::Duration;
use std::cell::Cell;
use std::pin::Pin;
use std::ops::{Generator, GeneratorState};

use redis::RedisResult;


pub trait Policy {
    fn get(&mut self, key: &str) -> String;
    fn set(&mut self, key: &str, value: &str);
    fn readSet(&mut self, s: String);
    fn get_readSet(&mut self) -> String;
}


// #[cfg(test)]
// mod test {
//     use super::*;
//     #[test]
//     fn get() {
//         let val = fetch_data();
//         println!("{}", val.unwrap());
//     }
// }