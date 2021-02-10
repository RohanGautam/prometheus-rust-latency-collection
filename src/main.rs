#[macro_use]
extern crate lazy_static;

use prometheus::{exponential_buckets, register_histogram_vec, HistogramVec, Registry};
use rand::prelude::*;
use std::result::Result;
use tokio::runtime::Runtime;
use warp::{Filter, Rejection, Reply};

lazy_static! {
    pub static ref THREAD_TIMES: HistogramVec = register_histogram_vec!(
        "thread_times",
        "Thread 1 process Times",
        &["thread_num", "batch_size"],
        exponential_buckets(0.005, 2.0, 20).unwrap()
    )
    .expect("metric can be created");
    // pub static ref THREAD_TIMES : IntGauge = IntGauge::new("time_taken", "measure the thead 1 processing tasks over time").expect("metric can be created");
    pub static ref REGISTRY: Registry = Registry::new();
}

// #[tokio::main]
fn main() {
    let t1 = std::thread::spawn(move || {
        let mut rng = thread_rng();
        loop {
            let start = std::time::Instant::now();
            std::thread::sleep(std::time::Duration::from_millis(rng.gen_range(200, 400)));
            let duration: u128 = start.elapsed().as_millis();
            track_request_time(duration, "1", rng.gen_range(1, 6));
            println!("thread 1 {}", duration);
        }
    });
    let t2 = std::thread::spawn(move || {
        let mut rng = thread_rng();
        loop {
            let start = std::time::Instant::now();
            std::thread::sleep(std::time::Duration::from_millis(
                rng.gen_range(10000, 20000),
            ));
            let duration: u128 = start.elapsed().as_millis();
            track_request_time(duration, "2", rng.gen_range(1, 6));
            println!("thread 2");
        }
    });
    let t3 = std::thread::spawn(move || {
        let mut rng = thread_rng();
        loop {
            let start = std::time::Instant::now();
            std::thread::sleep(std::time::Duration::from_millis(rng.gen_range(1000, 3000)));
            let duration: u128 = start.elapsed().as_millis();
            track_request_time(duration, "3", rng.gen_range(1, 6));
            println!("thread 3");
        }
    });

    let server_task = async move {
        register_custom_metrics();
        let metrics_route = warp::path!("api" / "prometheus").and_then(metrics_handler);
        println!("Started on port 8081");
        warp::serve(metrics_route).run(([127, 0, 0, 1], 8081)).await;
    };
    let server = std::thread::spawn(move || {
        let mut rt = Runtime::new().unwrap();
        rt.block_on(server_task);
    });
    let _ = server.join();
    let _ = t1.join();
    let _ = t2.join();
    let _ = t3.join();
}

fn register_custom_metrics() {
    REGISTRY
        .register(Box::new(THREAD_TIMES.clone()))
        .expect("collector can be registered");
}

fn track_request_time(response_time: u128, thread_num: &str, batch_size: i32) {
    THREAD_TIMES
        .with_label_values(&[thread_num, &batch_size.to_string()])
        .observe(response_time as f64);
}

async fn metrics_handler() -> Result<impl Reply, Rejection> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        eprintln!("could not encode custom metrics: {}", e);
    };
    let mut res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
        eprintln!("could not encode prometheus metrics: {}", e);
    };
    let res_custom = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("prometheus metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    res.push_str(&res_custom);
    Ok(res)
}
