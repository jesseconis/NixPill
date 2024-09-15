use structopt::StructOpt;
use walkdir::WalkDir;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, Read, Write};
use sha2::{Sha256, Digest};
use std::path::{PathBuf, Path};
use log::{info, debug, error};
use env_logger;
use std::time::Instant;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;
use num_cpus;
use tokio::io::AsyncReadExt;
use futures::stream::{self, StreamExt};

#[derive(StructOpt, Debug)]
#[structopt(name = "list_files")]
struct Opt {
    /// The path to list files from
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(long, default_value = "8192")]
    buffer_size: usize,

    /// Choose the implementation to use: "sequential", "rayon", "threadpool", "async"
    #[structopt(long, default_value = "rayon", possible_values = &["sequential", "rayon", "threadpool", "async"])]
    implementation: Implementation,

    /// Compare all implementations
    #[structopt(long)]
    compare: bool,
}

#[derive(Debug)]
enum Implementation {
    Sequential,
    Rayon,
    ThreadPool,
    Async,
}

impl std::str::FromStr for Implementation {
    type Err = String;

    fn from_str(s: &str) -> Result<Implementation, Self::Err> {
        match s.to_lowercase().as_str() {
            "sequential" => Ok(Implementation::Sequential),
            "rayon" => Ok(Implementation::Rayon),
            "threadpool" => Ok(Implementation::ThreadPool),
            "async" => Ok(Implementation::Async),
            _ => Err(format!("Invalid implementation: {}", s)),
        }
    }
}

fn setup_logging(verbose: u8) {
    let log_level = match verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .init();
}

fn collect_file_paths(dir: &PathBuf) -> Vec<PathBuf> {
    debug!("Collecting file paths from: {}", dir.display());
    let files: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| {
            match entry {
                Ok(e) if e.file_type().is_file() => {
                    let path = e.path();
                    Some(path.to_path_buf())
                },
                Ok(_) => None,
                Err(err) => {
                    error!("Error accessing entry: {}", err);
                    None
                }
            }
        })
        .collect();

    info!("Collected {} files.", files.len());
    files
}

fn compute_hash_with_buffer<P: AsRef<Path>>(path: P, buffer_size: usize) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; buffer_size];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

async fn compute_hash_with_buffer_async<P: AsRef<Path>>(path: P, buffer_size: usize) -> io::Result<String> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; buffer_size];

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

fn process_files_sequential(files: &Vec<PathBuf>, buffer_size: usize) {
    debug!("Processing {} files sequentially", files.len());
    for path in files.iter() {
        match compute_hash_with_buffer(path, buffer_size) {
            Ok(hash) => println!("{}  {}", hash, path.display()),
            Err(e) => error!("Error processing {}: {}", path.display(), e),
        }
    }
    info!("Finished processing all files sequentially");
}

fn process_files_rayon(files: &Vec<PathBuf>, buffer_size: usize) {
    debug!("Processing {} files with Rayon", files.len());
    files.par_iter().for_each(|path| {
        match compute_hash_with_buffer(path, buffer_size) {
            Ok(hash) => println!("{}  {}", hash, path.display()),
            Err(e) => error!("Error processing {}: {}", path.display(), e),
        }
    });
    info!("Finished processing all files with Rayon");
}

fn process_files_threadpool(files: &Vec<PathBuf>, buffer_size: usize) {
    debug!("Processing {} files using threadpool", files.len());
    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = channel();

    for path in files.iter() {
        let tx = tx.clone();
        let path = path.clone();
        pool.execute(move || {
            match compute_hash_with_buffer(&path, buffer_size) {
                Ok(hash) => tx.send((hash, path)).unwrap(),
                Err(e) => error!("Error processing {}: {}", path.display(), e),
            }
        });
    }
    drop(tx);

    for (hash, path) in rx {
        println!("{}  {}", hash, path.display());
    }

    info!("Finished processing all files using threadpool");
}

async fn process_files_async(files: &Vec<PathBuf>, buffer_size: usize) {
    debug!("Processing {} files asynchronously", files.len());

    // Get the system's file descriptor limit
    let fd_limit = get_fd_limit();

    // Subtract a safety margin to account for other open files
    let safety_margin = 100; // Adjust as necessary
    let max_concurrent = if fd_limit > safety_margin {
        fd_limit - safety_margin
    } else {
        fd_limit
    };

    info!("Using max_concurrent = {}", max_concurrent);

    stream::iter(files)
        .map(|path| {
            let buffer_size = buffer_size;
            let path = path.clone();
            async move {
                match compute_hash_with_buffer_async(&path, buffer_size).await {
                    Ok(hash) => println!("{}  {}", hash, path.display()),
                    Err(e) => error!("Error processing {}: {}", path.display(), e),
                }
            }
        })
        .buffer_unordered(max_concurrent as usize)
        .for_each(|_| async {})
        .await;

    info!("Finished processing all files asynchronously");
}

#[cfg(unix)]
fn get_fd_limit() -> u64 {
    use libc::{rlimit, RLIMIT_NOFILE, getrlimit};
    let mut limit = rlimit { rlim_cur: 0, rlim_max: 0 };
    unsafe {
        if getrlimit(RLIMIT_NOFILE, &mut limit) == 0 {
            limit.rlim_cur
        } else {
            error!("Failed to get file descriptor limit. Using default 1024.");
            1024
        }
    }
}

#[cfg(windows)]
fn get_fd_limit() -> u64 {
    // Windows doesn't have a strict limit like Unix. Set a high default.
    8192
}

#[cfg(not(any(unix, windows)))]
fn get_fd_limit() -> u64 {
    // Fallback for other platforms
    1024
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    setup_logging(opt.verbose);

    let files = collect_file_paths(&opt.path);

    if opt.compare {
        // Run each implementation and record execution times
        let mut results = Vec::new();

        let implementations = vec![
            ("Sequential", Implementation::Sequential),
            ("Rayon", Implementation::Rayon),
            ("ThreadPool", Implementation::ThreadPool),
            ("Async", Implementation::Async),
        ];

        for (name, imp) in implementations {
            let start = Instant::now();
            match imp {
                Implementation::Sequential => process_files_sequential(&files, opt.buffer_size),
                Implementation::Rayon => process_files_rayon(&files, opt.buffer_size),
                Implementation::ThreadPool => process_files_threadpool(&files, opt.buffer_size),
                Implementation::Async => process_files_async(&files, opt.buffer_size).await,
            }
            let duration = start.elapsed();
            results.push((name, duration));
        }

        // Write results to file
        let mut file = File::create("results.txt").expect("Unable to create results.txt");
        for (name, duration) in results {
            writeln!(file, "{}: {:?}", name, duration).expect("Unable to write to results.txt");
        }
    } else {
        let start = Instant::now();
        match opt.implementation {
            Implementation::Sequential => process_files_sequential(&files, opt.buffer_size),
            Implementation::Rayon => process_files_rayon(&files, opt.buffer_size),
            Implementation::ThreadPool => process_files_threadpool(&files, opt.buffer_size),
            Implementation::Async => process_files_async(&files, opt.buffer_size).await,
        }
        if opt.verbose > 0 {
            let duration = start.elapsed();
            info!("Execution time: {:?}", duration);
        }
    }
}