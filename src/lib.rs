extern crate lazy_static;
use std::sync::Mutex;
use std::time::{Instant, Duration};
use std::collections::LinkedList;

use std::fs;

lazy_static::lazy_static! {
    static ref PROGRAM_START: Mutex<Instant> = Mutex::new(Instant::now());
    static ref TRACE_BUFFER: Mutex<LinkedList<(Duration, Duration, &'static str)>> = Mutex::new(LinkedList::new());
}

use ctor::ctor;
#[ctor]
fn init() {
}

use ctor::dtor;
#[dtor]
fn deinit() {
    let mut result = "{\n\t\"traceEvents\": [ \n".to_string();
    for (time_stamp, duration, name) in TRACE_BUFFER.lock().unwrap().iter() {
        result += "\t{ \"pid\":1, \"tid\":1, \"ts\":";
        result += &time_stamp.as_micros().to_string();
        result += ", \"dur\":";
        result += &duration.as_micros().to_string();
        result += ", \"ph\":\"X\", \"name\":\"";
        result += name;
        result += "\" },\n";
    }
    result += "\n\t{ \"pid\":1, \"tid\":1, \"ts\":0, \"dur\":0, \"ph\":\"X\", \"name\":\"dummy\" }\n\t],\n\t\"meta_user\": \"aras\",\n\t\"meta_cpu_count\": \"8\"\n}";
    fs::write("trace.json", result).expect("Unable to write file");
}

pub struct TraceScope {
    pub name: &'static str,
    pub start_time: Instant,
}

impl TraceScope {
    pub fn new(name: &'static str) -> TraceScope {
        let start_time = Instant::now();
        TraceScope { name, start_time }
    }
}

impl Drop for TraceScope {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        let time_stamp = self.start_time - *(PROGRAM_START.lock().unwrap());
        let mut trace_buffer = TRACE_BUFFER.lock().unwrap();
        trace_buffer.push_back((time_stamp, duration, self.name));
    }
}

#[macro_export]
macro_rules! trace {
    ($name:expr) => {
        let _trace_scope = simple_tracing::TraceScope::new(&($name));
    };
    () => {
        let _trace_scope = simple_tracing::TraceScope::new(&(function!()));
    };
}

#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }}
}

