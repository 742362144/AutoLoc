#![feature(in_band_lifetimes)]
#![feature(generators, generator_trait)]

use std::sync::mpsc::Sender;
use std::pin::Pin;
use std::ops::{Generator, GeneratorState};
use std::sync::{Mutex, Arc};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::thread;
use std::time::Duration;
use libloading::os::unix::{Library, Symbol};

use rustlearn::prelude::*;
use rustlearn::datasets::iris;
use rustlearn::cross_validation::CrossValidation;
use rustlearn::linear_models::sgdclassifier::Hyperparameters;
use rustlearn::metrics::accuracy_score;
use serde_json::{Value, Map};
use std::collections::{HashSet, HashMap};

use md5::compute;

// extern crate hex;
// use openssl::aes::{AesKey, KeyError, aes_ige};
// use openssl::symm::Mode;
// use hex::{FromHex, ToHex};

use crate::policy::Policy;
use crate::cycles::rdtsc;

pub fn init(args: Arc<Map<String, Value>>, policy: Arc<Mutex<Policy>>) -> Pin<Box<Generator<Yield=u64, Return=u64> + 'static>> {
// pub fn init() -> Pin<Box<Generator<Yield=u64, Return=u64> + 'static>> {
//     print_hello();
//     println!("{}", "enter");
//     let ctx = tctx.clone();
//     let tx = ctx.lock();

    println!("{}", "gen init");
    let mut p = policy.clone();
    Box::pin(move || {
        let i: u64 = 1;
        p.lock().unwrap().set("A", "111");
        yield i;
        let mut j = 0;
        while j < 20 {
            p.lock().unwrap().get("A");
            j = j + 1;
        }

        1111
    })
}

pub fn khop(args: Arc<Map<String, Value>>, policy: Arc<Mutex<Policy>>) -> Pin<Box<Generator<Yield=u64, Return=u64> + 'static>> {
    // println!("{}", "gen khop");
    Box::pin(move || {
        let mut db_time: u64 = 0;
        let mut p = policy.clone();
        let params = args.clone();
        let nid_v = params.get("nodeId").unwrap();
        let nid = nid_v.to_string();
        let i: u64 = 1;
        // p.lock().unwrap().set("aaa", "1111");
        // println!("{}", sid);
        let obj = p.lock().unwrap().get(nid.as_str());
        db_time += obj.len() as u64;
        // println!("{} is empty", obj);
        if obj.is_empty() {
            return 1111;
        }
        // println!("{}", obj);
        let parsed: Value = serde_json::from_str(obj.as_str()).unwrap();
        let node: Map<String, Value> = parsed.as_object().unwrap().clone();
        let node_v1 = node.get("neigh").unwrap().clone();
        if node_v1.as_array().is_none() {
            return 1111;
        }
        let neigh = node_v1.as_array().unwrap();

        let mut traveled = HashSet::<String>::new();
        let mut next = HashSet::<String>::new();
        traveled.insert(nid);

        for n in neigh {
            let s = n.as_str().unwrap();
            next.insert(n.to_string());
        }
        // println!("neigh: {:?}", neigh);
        yield db_time;
        let mut j: i32 = 0;
        while j < 5 {
            let mut db_time: u64 = 0;
            let mut new_next = next.clone();
            next.clear();
            for n in new_next {
                if traveled.contains(n.as_str()) {
                    continue;
                }

                let obj = p.lock().unwrap().get(n.as_str());
                db_time += obj.len() as u64;
                if obj.is_empty() {
                    continue;
                }
                // println!("{}", obj);
                traveled.insert(n);
                let parsed_v: Value = serde_json::from_str(obj.as_str()).unwrap();
                let neigh_v = parsed_v.get("neigh").unwrap();
                let neigh = neigh_v.as_array().unwrap();
                // println!("neigh: {:?}", neigh);

                for nn in neigh {
                    if !traveled.contains(nn.as_str().unwrap()) {
                        next.insert(nn.to_string());
                    }
                }
            }
            j = j + 1;
            yield db_time;
        }
        // println!("traveled: {:?}", traveled);
        1111
    })
}

pub fn md5(args: Arc<Map<String, Value>>, policy: Arc<Mutex<Policy>>) -> Pin<Box<Generator<Yield=u64, Return=u64> + 'static>> {
    // println!("{}", "enter");
    // let ctx = tctx.clone();
    // let tx = ctx.lock();
    // println!("{}", "gen md5");
    let mut p = policy.clone();
    let sv = args.get("readSet").unwrap();
    let mut s: String = sv.to_string();
    Box::pin(move || {
        let mut db_time: u64 = 0;
        if s.is_empty() || s == "\"\"" {
            s = p.lock().unwrap().get("md5");
            policy.lock().unwrap().readSet(s.clone());
        }
        db_time += s.len() as u64;
        yield db_time;
        let mut j: i32 = 0;
        while j < 512 {
            let digest = compute(s.as_bytes());
            j += 1;
            yield 0;
        }
        1111
    })
}

pub fn pagerank(args: Arc<Map<String, Value>>, policy: Arc<Mutex<Policy>>) -> Pin<Box<Generator<Yield=u64, Return=u64> + 'static>> {
    // println!("{}", "gen khop");
    Box::pin(move || {
        let mut db_time: u64 = 0;
        let mut p = policy.clone();

        let obj = p.lock().unwrap().get("pagerank");
        db_time += obj.len() as u64;
        // println!("obj is {}", obj);
        if obj.is_empty() {
            return 1111;
        }
        let parsed: Value = serde_json::from_str(obj.as_str()).unwrap();
        let mut neigh = parsed.as_array().unwrap();
        let mut linked_node_map = HashMap::<u64, HashSet<u64>>::new();
        let mut PR_map = HashMap::<u64, f64>::new();
        // init graph
        for n in neigh {
            let s = n.as_array().unwrap();
            let from = s[0].as_u64().unwrap();
            let to = s[1].as_u64().unwrap();
            if !linked_node_map.contains_key(&from) {
                let mut set = HashSet::<u64>::new();
                linked_node_map.insert(from, set);
                PR_map.insert(from, 0.0);
            }
            if !linked_node_map.contains_key(&to) {
                let mut set = HashSet::<u64>::new();
                linked_node_map.insert(to, set);
                PR_map.insert(to, 0.0);
            }
            let mut set = linked_node_map.get_mut(&from).unwrap();
            set.insert(to);
        }

        let epoch_num: i32 = 5;
        let d: f64 = 0.85;
        yield db_time;
        let mut j: i32 = 0;
        while j < epoch_num {
            let mut i = 0;
            while i < 10 {
                let obj = p.lock().unwrap().get("pagerank");
                i = i + 1;
            }
            let mut db_time: u64 = 0;
            let mut map = HashMap::<u64, f64>::new();
            for (node, pr) in PR_map {
                let mut score = 0.0;
                let neighs = linked_node_map.get_mut(&node).unwrap();
                for neigh in neighs.iter() {
                    score += pr;
                }

                map.insert(node.clone(), (1.0 - d) + d * score);
            }
            PR_map = map;
            j = j + 1;
            yield db_time;
        }
        // println!("{:?}", PR_map.values());
        1111
    })
}

// pub fn aes(policy: Arc<Mutex<Policy>>) -> Pin<Box<Generator<Yield=u64, Return=u64> + 'static>> {
//     // println!("{}", "enter");
//     // let ctx = tctx.clone();
//     // let tx = ctx.lock();
//
//     println!("{}", "gen aes");
//     let mut p = policy.clone();
//     Box::pin(move || {
//         let i:u64 = 1;
//         let raw_key = "000102030405060708090A0B0C0D0E0F";
//         let hex_cipher = "12345678901234561234567890123456";
//         let randomness = "000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F";
//         yield i;
//
//         if let (Ok(key_as_u8), Ok(cipher_as_u8), Ok(mut iv_as_u8)) =
//         (Vec::from_hex(raw_key), Vec::from_hex(hex_cipher), Vec::from_hex(randomness)) {
//             let key = AesKey::new_encrypt(&key_as_u8)?;
//             let mut output = vec![0u8; cipher_as_u8.len()];
//             aes_ige(&cipher_as_u8, &mut output, &key, &mut iv_as_u8, Mode::Encrypt);
//             // assert_eq!(output.to_hex(), "a6ad974d5cea1d36d2f367980907ed32");
//         }
//         1111
//     })
// }


// pub fn rg(policy: Arc<Mutex<Policy>>) -> Pin<Box<Generator<Yield=u64, Return=u64> + 'static>> {
//     let mut p = policy.clone();
//     Box::pin(move || {
//         let (X, y) = iris::load_data();
//
//         let num_splits = 10;
//         let num_epochs = 5;
//
//         let mut accuracy = 0.0;
//
//         for (train_idx, test_idx) in CrossValidation::new(X.rows(), num_splits) {
//
//             let X_train = X.get_rows(&train_idx);
//             let y_train = y.get_rows(&train_idx);
//             let X_test = X.get_rows(&test_idx);
//             let y_test = y.get_rows(&test_idx);
//
//             let mut model = Hyperparameters::new(X.cols())
//                 .learning_rate(0.5)
//                 .l2_penalty(0.0)
//                 .l1_penalty(0.0)
//                 .one_vs_rest();
//
//             for _ in 0..num_epochs {
//                 model.fit(&X_train, &y_train).unwrap();
//             }
//
//             let prediction = model.predict(&X_test).unwrap();
//             accuracy += accuracy_score(&y_test, &prediction);
//         }
//
//         accuracy /= num_splits as f32;
//
//         1111
//     })
//
//
//
//
//
//
// }


// pub fn init(policy: Arc<Mutex<Policy>>) -> Pin<Box<Generator<Yield=u64, Return=u64> + 'static>> {
//     // println!("{}", "test");
//     // let mut ctx = ThreadSafeContext::new();
//     // let b = Arc::new(&ctx);
//     // // type Proc = unsafe extern "C" fn(Rc<Db>) -> Pin<Box<Generator<Yield=u64, Return=InvokeResult>>>;
//     // type Proc = unsafe extern "C" fn(Arc<&ThreadSafeContext<DetachedFromClient>>) -> Pin<Box<Generator<Yield=u64, Return=u64>>>;
//
//     type Proc = unsafe extern fn(Arc<Mutex<Policy>>) -> Pin<Box<Generator<Yield=u64, Return=u64>>>;
//     // type Proc = unsafe extern "C" fn(Arc<Mutex<Policy>>) -> Pin<Box<Generator<Yield=u64, Return=u64>>>;
//     let library_path = String::from("/home/coder/IdeaProjects/add/target/debug/libadd.so");
//     println!("Loading add() from {}", library_path);
//
//     let lib = Library::new(library_path).unwrap();
//
//     unsafe {
//         let func: Symbol<Proc> = lib.get(b"init").unwrap();
//         func(policy)
//     }
// }