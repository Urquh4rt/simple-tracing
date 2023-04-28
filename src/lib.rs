extern crate lazy_static;
use std::sync::Mutex;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use itertools::Itertools;

use std::fs;

lazy_static::lazy_static! {
    static ref START_TIME: Instant = Instant::now();
    static ref EVENT_COUNTER: Mutex<u32> = Mutex::new(0);
    static ref TRACE_BUFFER: Mutex<HashMap<u32, (Duration, Duration, &'static str)>> = Mutex::new(HashMap::new());
    static ref AGGREGATED_DURATIONS: Mutex<HashMap<&'static str, Duration>> = Mutex::new(HashMap::new());
}

use ctor::ctor;
#[ctor]
fn init() {
}

use ctor::dtor;
#[dtor]
fn deinit() {
    let mut result = "{\n\t\"traceEvents\": [ \n".to_string();
    let aggregated_durations = AGGREGATED_DURATIONS.lock().unwrap();
    let trace_buffer = TRACE_BUFFER.lock().unwrap();
    for (_, (time_stamp, duration, name)) in trace_buffer.iter().sorted() {
        result += "\t{ \"pid\":1, \"tid\":1, \"ts\":";
        result += &time_stamp.as_micros().to_string();
        result += ", \"dur\":";
        result += &duration.as_micros().to_string();
        result += ", \"ph\":\"X\", \"name\":\"";
        result += name;
        result += "\", \"args\":{ \"aggregated_duration\":";
        result += &aggregated_durations.get(name).unwrap().as_micros().to_string();
        result += "} },\n";
    }
    result += "\n\t{ \"pid\":1, \"tid\":1, \"ts\":0, \"dur\":0, \"ph\":\"X\", \"name\":\"dummy\" }\n\t],\n\t\"meta_user\": \"aras\",\n\t\"meta_cpu_count\": \"8\"\n}";
    fs::write("trace.json", result).expect("Unable to write file");
}

pub struct TraceScope {
    pub event_id: u32,
    pub name: &'static str,
    pub time_stamp: Duration,
}

impl TraceScope {
    pub fn new(name: &'static str) -> TraceScope {
        TraceScope { event_id: {
                let mut event_counter = EVENT_COUNTER.lock().unwrap();
                *event_counter += 1;
                *event_counter
            }, name, time_stamp: START_TIME.elapsed()
        }
    }
}

impl Drop for TraceScope {
    fn drop(&mut self) {
        let duration = START_TIME.elapsed() - self.time_stamp;
        let mut trace_buffer = TRACE_BUFFER.lock().unwrap();
        trace_buffer.insert(self.event_id, (self.time_stamp, duration, self.name));

        let mut aggregated_durations = AGGREGATED_DURATIONS.lock().unwrap();
        let aggregated_duration = aggregated_durations.entry(self.name).or_insert(Duration::new(0, 0));
        *aggregated_duration += duration;
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

