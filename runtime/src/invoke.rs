use std::sync::Mutex;
use std::sync::mpsc::Sender;

pub struct Invoke{
    pub tx: Mutex<Sender<String>>,  // send result
    pub req: String,
    pub readSet: String,
}