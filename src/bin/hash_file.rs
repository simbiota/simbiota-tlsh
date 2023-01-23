use std::env::args;
use std::io::Read;
use std::path::Path;
use simbiota_tlsh::TLSHBuilder;

fn main() {
    let all_args: Vec<String> = args().collect();
    let mut debug_mode = false;
    if all_args.contains(&"-d".to_string()) {
        debug_mode = true;
    }

    let path = args().nth(1).unwrap();
    let mut file = std::fs::File::open(Path::new(&path)).unwrap();

    let mut buffer: [u8; 1024] = [0; 1024];
    let mut builder = TLSHBuilder::default();
    while let Ok(read) = file.read(&mut buffer) {
        if read > 0 {
            builder.update(&buffer[0..read]);
        }else {
            break;
        }
    }
    builder.finalize();
    println!("{}", builder.get_hashes().first().expect("no hash").expect("hashing error").to_digest());
    if debug_mode {
        let hash = builder.get_hashes().first().unwrap().unwrap();
        println!("{:#?}", hash);
    }
}