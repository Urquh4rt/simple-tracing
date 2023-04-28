extern crate lazy_static;
use std::sync::Mutex;
use std::time::{Instant};

use std::fs;

lazy_static::lazy_static! {
    static ref PROGRAM_START: Mutex<Instant> = Mutex::new(Instant::now());
    static ref TRACE_BUFFER: Mutex<String> = Mutex::new("{\n\t\"traceEvents\": [ \n".to_string());
}

use ctor::ctor;
#[ctor]
fn init() {
}

use ctor::dtor;
#[dtor]
fn deinit() {
    let mut result = TRACE_BUFFER.lock().unwrap().to_string();
    result += "\n\t{ \"pid\":1, \"tid\":1, \"ts\":0, \"dur\":0, \"ph\":\"X\", \"name\":\"dummy\" }\n\t],\n\t\"meta_user\": \"aras\",\n\t\"meta_cpu_count\": \"8\"\n}";
    // println!("{}", result);

    fs::write("trace.json", result).expect("Unable to write file");
}

pub struct TraceScope {
    pub name: String,
    pub start_time: Instant,
}

impl TraceScope {
    pub fn new(name: String) -> TraceScope {
        let start_time = Instant::now();
        TraceScope { name, start_time }
    }
}

impl Drop for TraceScope {
    fn drop(&mut self) {
        let end_time = Instant::now();
        let duration = end_time - self.start_time;
        let time_stamp = self.start_time - *(PROGRAM_START.lock().unwrap());
        let mut trace_buffer = TRACE_BUFFER.lock().unwrap();
        *trace_buffer += "\t{ \"pid\":1, \"tid\":1, \"ts\":";
        *trace_buffer += &time_stamp.as_micros().to_string();
        *trace_buffer += ", \"dur\":";
        *trace_buffer += &duration.as_micros().to_string();
        *trace_buffer += ", \"ph\":\"X\", \"name\":\"";
        *trace_buffer += &self.name;
        *trace_buffer += "\" },\n";
    }
}

#[macro_export]
macro_rules! trace {
    ($name:expr) => {
        let _trace_scope = simple_tracing::TraceScope::new($name.to_string());
    };
    () => {
        let _trace_scope = simple_tracing::TraceScope::new(function!().to_string());
    };
}

// get the name of the current function
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

