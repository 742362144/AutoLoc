extern crate runtime;

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use std::sync::mpsc::{Sender, Receiver};
use std::time::Duration;
use std::pin::Pin;
use runtime::task::{Task, TaskState};
use runtime::invoke::{Invoke};

use runtime::policy::Policy;
use redis::{Commands, Connection, RedisResult, RedisError};


pub struct LocalPolicy {
    pub con: Connection,
    pub cost: u64,
    pub readSet: String,
}

impl LocalPolicy {
    pub fn new(con: Connection) -> LocalPolicy {
        LocalPolicy {
            con,
            cost: 0,
            readSet: String::from(""),
        }
    }
    pub fn clear(&mut self) {
        self.cost = 0;
    }

    pub fn time(&mut self) -> u64 {
        self.cost
    }
}

impl Policy for LocalPolicy {
    fn get(&mut self, key: &str) -> String {
        let mut s1 = String::from(key);
        if key.contains("\"") {
            s1.remove(0);
            s1.remove(s1.len() - 1);
        };

        // println!("key is {}", s1);
        // println!("key len {}", s1.len());
        // let s2 = String::from("999");
        // println!("key is {}", s2);
        // println!("key len {}", s2.len());

        let f: RedisResult<String> = self.con.get(s1.as_str());
        match f {
            Ok(rv) => {
                self.cost += rv.len() as u64;
                // println!("{}", rv);
                rv
            }
            Err(error) => {
                // println!("{}", error.to_string());
                String::from("")
            }
        }


        // // let res = self.con.get(key)?;
        // let res = self.con.get(String::from(key))?;
        // match res { //判断方法结果
        //     Ok(val) => {
        //         println!("{}", val);
        //         RedisResult::Ok(val)
        //     } //OK 代表读取到文件内容，正确打印文件内容
        //     Err(e) => {
        //         println!("{}", e);
        //         RedisResult::Ok("Err")
        //     } //Err代表结果不存在，打印错误结果
        // }
    }

    fn set(&mut self, key: &str, value: &str) {
        // println!("{}", "set");
        let _: () = self.con.set(key, value).unwrap();
    }


    fn readSet(&mut self, s: String){
        self.readSet = s;
    }

    fn get_readSet(&mut self) -> String {
        self.readSet.clone()
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_init() {
        let con = get_con();
        let mut policy = LocalPolicy::new(con);
        assert!(!policy.get("1").is_empty());
    }
}


// fn main() {
//     let res = fetch_data();
//     match res { //判断方法结果
//         Ok(val) => { println!("{}", val) } //OK 代表读取到文件内容，正确打印文件内容
//         Err(e) => { println!("{}", e) } //Err代表结果不存在，打印错误结果
//     }
//     // println!("{}", res.unwrap());
// }

fn fetch_data() -> redis::RedisResult<isize> {
    // connect to redis
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;
    // throw away the result, just make sure it does not fail
    let _: () = con.set("my_key", 42)?;
    // read back the key and return it.  Because the return value
    // from the function is a result for integer this will automatically
    // convert into one.
    con.get("my_key")
}

pub fn get_con() -> Connection {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();

    con
}
