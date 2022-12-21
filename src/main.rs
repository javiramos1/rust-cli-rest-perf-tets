use clap::Parser;
use reqwest::{Response, Client};
use serde::Serialize;
use std::error::Error;
use std::fmt::Debug;
use std::thread;
use tracing::{debug, info,Level, error};
use tracing_subscriber::FmtSubscriber;
use tokio::task;
use tokio::task::JoinHandle;
use std::time::Instant;
use std::sync::{Arc};
use rand::prelude::*;
use tokio::time::{sleep, Duration};
use cli_table::{ print_stdout, Table, WithTitle};

#[derive(Parser, Debug, Clone)]
#[clap(name = "Rust HTTP Performance Test")]
#[clap(author = "Javier Ramos")]
#[clap(version = "1.0")]
#[clap(about = "Rust CLI using clap library which send HTTP requests that can be used for performance test", long_about = None)]
struct Args {

    #[clap(value_parser, help = "HTTP method")]
    method: String,

    #[clap(value_parser, help = "Target URL")]
    url: String,

    #[clap(short, long, default_value_t = 1, help = "Number of Producers sending requests")]
    producers: u32,

    #[clap(short, long, default_value_t = 200, help = "Expected HTTP Return Status")]
    expected_status: u16,

    #[clap(short, long, default_value_t = 1000, help = "Number of Request to send")]
    requests: u32,

    #[clap(short, long, help = "Body for HTTP POST requests")]
    body: Option<String>,

    #[clap(short, long, default_value_t = 0, help = "Wait time in milliseconds between requests")]
    throttle_ms: u32,

    #[clap(short, long, default_value_t = -1, help = "Ramp up delay for each producer in milliseconds, this is the maximum time, a number between zero and this one will be selected. Default is the number of producers, set 0 to disable")]
    max_ramp_up_time: i32,

}

#[derive(Table, Debug)]
struct TestResult {
    #[table(title = "Elapsed Time")]
    time: String,
    #[table(title = "Avg Request Time")]
    avg_request_time: String,
    #[table(skip)]
    avg_request_time_mills: u128,
    #[table(title = "Requests")]
    total_requests: u128,
    #[table(title = "Failed")]
    failed_request: u128,
    #[table(title = "Fail %")]
    fail_ratio: f32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let args = Args::parse();

    debug!("Arg: {:?}", args);

    info!("Starting performance test with {} requests. Url: {}", args.requests, args.url);

    let now = Instant::now();

    let mut handlers: Vec<JoinHandle<_>> = Vec::new();

    let max_ramp_speed = if args.max_ramp_up_time >= 0 {args.max_ramp_up_time} else {args.producers as i32 * 1000 };
    let mut rng = rand::thread_rng();
    let request_num = args.requests / args.producers;

    for i in 0..args.producers {
        let wait_time = match args.producers {
            1 => 0,
            _ => if max_ramp_speed == 0 { 0 } else {rng.gen_range(0..max_ramp_speed) as u64}
        };
            
        handlers.push(task::spawn(
            run_producer(args.clone(), request_num, wait_time, i)
        ));
    }

    info!("Waiting for producers to complete...");
    let mut total = 0;
    let mut failures = 0;
    let mut avg_total = 0;

    for thread in handlers {
        match thread.await? {
            Err(r) => error!("Producer Error {:?}", r),
            Ok(r) => {
                debug!("OK: {:?}", r);
                total = total + r.total_requests;
                failures = failures + r.failed_request;
                avg_total = avg_total + r.avg_request_time_mills;
            }
        }
    }

    let elapsed = now.elapsed();
    let total_time = elapsed.as_millis();
    let avg = avg_total/ args.producers as u128;
    let fail_ratio = (failures as f32 / total as f32 ) * 100 as f32;
    let time = if total_time > 1000 { format!("{:.2}s", total_time as f32 / 1000 as f32)} else {format!("{}ms", total_time)};

    let result = vec![TestResult {
        avg_request_time_mills: avg,
        avg_request_time : format!("{}ms", avg),
        fail_ratio: fail_ratio,
        failed_request: failures,
        time,
        total_requests: total
    }];

    println!("*************************************************************************");
    println!("******************* PERFORMANCE TESTS COMPLETED *************************");
    println!("*************************************************************************");
    print_stdout(result.with_title()).unwrap();

    Ok(())
}

async fn run_producer(args: Args, request_num: u32, wait_time: u64, index: u32)  -> Result<TestResult, Box<dyn Error + Sync + Send>> {
    debug!("run_producer: Producer {} sleep for {}.", index, wait_time);
    let now = Instant::now();
    sleep(Duration::from_millis(wait_time)).await;

    let c = reqwest::Client::builder()
    .pool_max_idle_per_host(0)
    .timeout(Duration::from_secs(900))
    .connect_timeout(Duration::from_secs(900))
    .build()?;
    let client = Arc::new(c);

    info!("run_producer: Producer {} Sending {} requests...", index, request_num);

    let mut handlers: Vec<JoinHandle<_>> = Vec::new();
    for _ in 0..request_num {
        debug!("Sleep for a: {}", args.throttle_ms);
        thread::sleep(Duration::from_millis(args.throttle_ms as u64));
        let async_ret = match args.method.as_str() {
            "POST" => task::spawn(post_request(client.clone(), args.url.clone(), args.body.clone())),
            _ => task::spawn(get_request(client.clone(), args.url.clone()))
        };
        handlers.push(async_ret);
    }
   
    let mut failures = 0;
    let mut total = 0;
    let mut avg_total = 0;
    debug!("Waiting for http calls to complete...");
    for thread in handlers {
         match thread.await? {
            Err(r) =>  {
                error!("HTTP Error {:?}", r);
                failures = failures + 1;
                total = total + 1;
            },
            Ok(r) => {
                debug!("OK: {:?}", r);
                if r.0.status().as_u16() != args.expected_status {
                    failures = failures + 1;
                    error!("HTTP Error {:?}", r);
                } 
                total = total + 1;
                avg_total = avg_total + r.1.as_millis();
            }
        }
    }

    avg_total = avg_total / total;

    let fail_ratio = (failures as f32 / total as f32 ) * 100 as f32;

    let elapsed = now.elapsed();

    info!("run_producer: Producer {} Requests {}. FAILURES: {}/{}. Time: {:.2?}", index, request_num, failures, total, elapsed);
    
    Ok(TestResult { 
        time: "".to_owned(),
        total_requests: total, 
        failed_request: failures, 
        fail_ratio, 
        avg_request_time_mills: avg_total,
        avg_request_time : "".to_owned()
    })
}

async fn post_request<T: Serialize + Debug>(client: Arc<Client>, url: String, item: T) -> Result<(Response, Duration), reqwest::Error> {
    debug!("Making POST call to {}. Payload: {:?}", url, item);
    let now = Instant::now();
    let ret =  client.post(url)
    .timeout(Duration::from_secs(900))
    .json(&item)
    .send().await?;

    let elapsed = now.elapsed();
    info!("run_producer: Producer {} Time: {:.2?}", 1, elapsed);
    return Ok((ret, elapsed));
}

async fn get_request(client: Arc<Client>, url: String) -> Result<(Response, Duration), reqwest::Error> {
    debug!("Making GET call to {}.", url);
    let now = Instant::now();
    let ret = client.get(url)
    .timeout(Duration::from_secs(900))
    .send().await?;

    let elapsed = now.elapsed();
    return Ok((ret, elapsed));
}

