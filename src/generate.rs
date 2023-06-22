use crate::*;
use libc::{c_int, cpu_set_t, CPU_SET, CPU_SETSIZE, sched_setaffinity, sched_getaffinity, pid_t, listen, CPU_ISSET, CPU_ZERO};
use num_cpus;
use std::mem::{zeroed, size_of};
use std::net::{TcpListener, SocketAddrV6};
use std::process::{Command, exit, Stdio, Child};
use std::thread;
// use thread_affinity::ThreaadAffinity;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use bincode::{serialize, deserialize};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
/// data for extraction
pub struct Data {
    skip_ecls: HashMap<String, f64>,
    grammar: HashMap<String, Vec<String>>,
}

fn deserialize_data(serialized_data: &[u8]) -> Result<Data, Box<dyn Error>> {
    match bincode::deserialize::<Data>(serialized_data) {
        Ok(data) => Ok(data),
        Err(err) => Err(Box::new(err)),
    }
}

fn send_data(skip_ecls: &HashMap<String, f64>, grammar: &HashMap<String, Vec<String>>) {    
    let data: Data = Data {
        skip_ecls: skip_ecls.clone(),
        grammar: grammar.clone(),
    };

    let serialized_data = bincode::serialize(&data).unwrap();
    // println!("{:?}", serialized_data);


    let data: Result<Data, Box<dyn std::error::Error>> = Ok(deserialize_data(&serialized_data).unwrap());
    // println!("{:?}", data);
}

/// ### private unsafe function to set process affinity
/// #### Arguments
/// * `pid` - process id
/// * `processor id` - processor id (CPU logic core id)
/// #### Return
/// * `c_int` - return 0 on success, return -1 on failure
unsafe fn set_proc_affinity(pid: pid_t, processor_id: usize) -> c_int {
    let mut cpuset: cpu_set_t = zeroed();
    CPU_SET(processor_id, &mut cpuset);
    sched_setaffinity(pid, size_of::<cpu_set_t>(), &cpuset)
}

/// ### public function to start extraction of a single expression
/// #### Argument
/// `cli` - pre-processed command line arguments
/// #### Return
/// `None`
pub fn generate_expr(cli: &mut Vec<CmdLineArg>) {
    /* initialize ctx_gr struct */
    let mut expr: &str = "";
    if let CmdLineArg::String(init_expr) = &cli[3] {
        expr = init_expr;
    }
    log_info(&format!("Expression: {}\n", expr));
    let mut ctx_gr = ContextGrammar::new(&expr);

    /* create egraph, skip_ecls, grammar, init_rewrite */
    ctx_gr.setup();
    let skip_ecls = Arc::new(ctx_gr.skip_ecls.clone());
    let grammar = Arc::new(ctx_gr.grammar.clone());
    let init_rw = &ctx_gr.init_rw.clone();
    println!("{:?}", init_rw);
    println!("{}", init_rw.len());

    /* get number of processes */
    let num_proc = init_rw.len();

    /* bind the parent process to tcp ports */
    let tcp_listeners: Vec<TcpListener> = (0..num_proc).map(|proc_idx| {
        let addr = format!("127.0.0.1:{}", 8000 + proc_idx);
        TcpListener::bind(&addr).unwrap_or_else(|_| {
            log_error(&format!("[ERROR]: Failed to bind IP address \"{}\"\n.", addr));
            exit(1)
        })
    }).collect();

    /* insert empty str for socket address & get CPU's number of logical cores */
    cli.push(CmdLineArg::String("".to_string()));
    let num_logical_cores = num_cpus::get();

    /* spawn children processes & set process affinity */
    let mut child_procs: Vec<Child> = init_rw.into_iter().zip(0..num_proc).map(|(rw, proc_idx)| {
        let addr = format!("127.0.0.1:{}", 8000 + proc_idx);
        cli[3] = CmdLineArg::String(rw.clone());
        cli[4] = CmdLineArg::String(addr.clone());
        let args: Vec<String> = cli.iter().map(|arg| arg.to_string()).collect();

        let child_proc = Command::new("../target/debug/multiproc")
                                    .args(&args)
                                    .spawn()
                                    .expect("[ERROR]: Failed to spawn child process.");

        let pid = child_proc.id() as pid_t;
        let processor_id = proc_idx % num_logical_cores;
        let ret = unsafe { set_proc_affinity(pid, processor_id) };
        match ret {
            0 => {
                log_debug(&format!("Set process {}'s process affinity to processor {}.\n", pid, processor_id));
            },
            _ => {
                log_error(&format!("Failed to set process {}'s process affinity to processor {}.\n", pid, processor_id));
                exit(1);
            },
        }

        child_proc
    }).collect();

    /* send data to all children processes through sockets */
    let handles: Vec<_> = tcp_listeners.into_iter().map(|listener| {
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        println!("New connection: {}", stream.peer_addr().unwrap());
                        thread::spawn(move|| {
                            // connection succeeded
                            // for i in (0..1000000).step_by(10) {
                                stream.write(b"Hello World!");
                                println!("send hello world!");
                            // }
                        });
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        /* connection failed */
                    }
                }
            }
        })
    }).collect();

    // for handle in handles {
    //     handle.join().unwrap();
    // }

    for child_proc in &mut child_procs {
        let pid = child_proc.id();
        child_proc.wait().expect(&format!("[ERROR]: Failed to wait for processor {}.\n", pid));
        let exit_status = child_proc.wait().expect("Failed to wait for child process.");
        let exit_code = exit_status.code();

        if let Some(exit_code) = exit_code {
            match exit_code {
                0 => { log_debug(&format!("Child process {} terminated successfully with an exit code {}.\n", pid, exit_code)); },
                _ => { log_error(&format!("Child process {} terminated with a non-zero exit code {}.\n", pid, exit_code)); },
            }
        }
    }

    // for listener in listeners {
        
    // }

    // let handles: Vec<_> = (0..num_proc).map(|proc_idx| {
    //     let addr = format!("127.0.0.1:{}", 8000 + proc_idx);
    //     let listener = TcpListener::bind(&addr)
    //         .expect(format!("[ERROR]: Failed to bind TCP listener with address {}.", &addr).as_str());
    //     let listener_addr = listener.local_addr().expect("[ERROR]: Failed to get local address.");


    //     let processor_id = proc_idx % num_logical_cores;

    //     cli[4] = CmdLineArg::String(listener_addr.to_string());
    //     let args: Vec<String> = cli.iter().map(|arg| arg.to_string()).collect();
    //     let skip_ecls_clone = Arc::clone(&skip_ecls);
    //     let grammar_clone = Arc::clone(&grammar);

    //     thread::spawn(move || {
    //         let mut child_proc = Command::new("../target/debug/multiproc").args(&args).spawn()
    //             .expect("[ERROR]: Failed to spawn process.");

    //         let pid = child_proc.id();

    //         if let Err(err) = set_proc_affinity(pid as pid_t, processor_id) {
    //             println!("Error setting affinity for process {}: {}", pid, err);
    //         } else {
    //             println!("Process {} affinity set to core {}", pid, processor_id);
    //         }

    //         send_data(&skip_ecls_clone, &grammar_clone);

    //         let exit_status = child_proc.wait().expect("Failed to wait for child process.");
    //         let exit_code = exit_status.code().unwrap_or(1);
    //         println!("Process {} finished with exit code {}", pid, exit_code);
    //     })
    // }).collect();

    // for handle in handles {
    //     handle.join().unwrap();
    // }

    println!("Generate Finished");
}

pub fn generate_file(input_filename: &str, output_filename: &str) {
    // Open the input file and create output file
    let input_file = File::open(input_filename)
        .expect(&format!("[ERROR]: Failed to open input file \"{}\".", input_filename));
    let output_file = File::create(output_filename)
        .expect(&format!("[ERROR]: Failed to create output file \"{}\".", output_filename));

    // Create buffered reader and writer for the input and output files
    let reader = BufReader::new(input_file);
    let mut writer = BufWriter::new(output_file);

    for expr in reader.lines() {
        let expr = expr.expect("[ERROR]: Error reading line from file.");

        log_info(&format!("Expression: {}\n", expr));
        let mut ctx_gr = ContextGrammar::new(&expr);

        /* create egraph, skip_ecls, grammar, init_rewrite */
        ctx_gr.setup();

        let root_ecls = &ctx_gr.root_ecls.clone();
        println!("{:?}", root_ecls);



        /* TODO: Start multiprocessing here */
        // Step 3. get the root-eclass id
        // Step 4. get all the root-enodes in root-eclass
        // Step 5. create # of processes based on
        //         # of root-enodes or # of CPUs
        // Step 6. create corresponding # of socket addresses
        // Step 7. create connections with children processes multi-threading
        // Step 8. send hyperparameters & hashmap to all children processes
    }
}

pub fn generate(args: &Vec<String>) {
    let mut cli = parse_args(&args);
    println!("{:?}", cli);
    if cli.len() == 4 {
        generate_expr(&mut cli);
    } 
    // else {
    //     let input_filename = cli.get("input_filename").unwrap();
    //     let output_filename = cli.get("output_filename").unwrap();
    //     generate_file(input_filename, output_filename);
    // }
}