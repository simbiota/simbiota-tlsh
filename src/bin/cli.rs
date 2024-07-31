/// A simple CLI interface for TLSH, mimicking the behaviour of the reference TLSH binary
use clap::{CommandFactory, Parser};
use std::path::{Path, PathBuf};
#[derive(Parser)]
struct Args {
    /*#[arg(short('c'))]
    pub compare: Option<String>,*/
    #[arg(short('f'))]
    pub file: Option<PathBuf>,
    /*  #[arg(short('d'))]
        pub digest: Option<String>,

        #[arg(short('T'))]
        pub threshold: Option<i32>,

        #[arg(short('r'))]
        pub recursive_dir: Option<PathBuf>,
    */
}

fn main() {
    let args = Args::parse();

    // determine mode
    match args.file {
        Some(file) => hash_file(file),
        _ => {
            Args::command().print_help().unwrap();
        }
    }
}

fn hash_file(file: impl AsRef<Path>) {
    let file_bytes = std::fs::read(file.as_ref()).unwrap();
    let mut builder = simbiota_tlsh::TLSHBuilder::new();
    builder.update(&file_bytes);
    builder.finalize();
    println!(
        "{}\t{}",
        builder.get_hash().unwrap().to_digest(),
        file.as_ref().display()
    );
}
