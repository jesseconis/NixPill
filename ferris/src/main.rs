use walkdir::WalkDir;
use structopt::StructOpt;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, Read};
use sha2::{Sha256, Digest};
use std::path::{PathBuf, Path};


#[derive(StructOpt, Debug)]
#[structopt(name = "list_files")]
struct Opt {
    /// The path to list files from
    #[structopt(parse(from_os_str))]
    path: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let dir = opt.path;

    // Collect all file paths
    let files: Vec<_> = WalkDir::new(&dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .collect();

    // Process files in parallel
    files.par_iter().for_each(|entry| {
        match compute_hash(entry.path()) {
            Ok(hash) => println!("{}  {}", hash, entry.path().display()),
            Err(e) => eprintln!("Error processing {}: {}", entry.path().display(), e),
        }
    });
}

fn compute_hash<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?;
    // Rest of the code...
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192]; // Read in chunks of 8KB

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


// fn compute_hash(path: &PathBuf) -> io::Result<String> {
//     let mut file = File::open(path)?;
//     let mut hasher = Sha256::new();
//     let mut buffer = [0u8; 8192]; // Read in chunks of 8KB
// 
//     loop {
//         let bytes_read = file.read(&mut buffer)?;
//         if bytes_read == 0 {
//             break;
//         }
//         hasher.update(&buffer[..bytes_read]);
//     }
// 
//     let result = hasher.finalize();
//     Ok(format!("{:x}", result))
// }
// 
