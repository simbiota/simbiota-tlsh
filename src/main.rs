mod builder;
mod diff;
mod digest;
mod tlsh;
mod util;

use crate::{
    builder::{TLSHBuilder, TLSHError},
    tlsh::TLSH,
};

enum Input {
    Data(Vec<u8>),
    Digest(String),
}

enum ColorSupplyState {
    Unspecified,
    DigestSupplied,
    Specified(u8),
}

fn main() {
    let mut inputs: Vec<Input> = vec![];
    let mut color = ColorSupplyState::Unspecified;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .unwrap_or_else(|| panic!("Supply value for option {} !", &arg));
        match arg.as_str() {
            "-c" => {
                color = ColorSupplyState::Specified(value.parse::<u8>().expect("Invalid color"));
            }
            "-f" => {
                inputs
                    .push(Input::Data(std::fs::read(&value).unwrap_or_else(|_| {
                        panic!("Failed reading file {} !", &value)
                    })));
            }
            "-d" => {
                inputs.push(Input::Digest(value));
                if let ColorSupplyState::Unspecified = color {
                    color = ColorSupplyState::DigestSupplied;
                }
            }
            option => panic!("Unknown option {option}"),
        }
    }
    if let ColorSupplyState::Unspecified = color {
        color = ColorSupplyState::Specified(0);
    }

    let mut hashes: Vec<Option<TLSH>> =
        std::iter::repeat_with(|| None).take(inputs.len()).collect();

    for _ in 0..2 {
        for (hash, input) in hashes.iter_mut().zip(&inputs) {
            if hash.is_none() {
                match (&color, input) {
                    (ColorSupplyState::Specified(v), Input::Data(ref data)) => {
                        let mut builder = TLSHBuilder::new(&[*v]);
                        builder.update(data);
                        builder.finalize();
                        *hash = match builder.get_hashes()[0] {
                            Ok(tlsh) => Some(tlsh),
                            Err(TLSHError::Length) => panic!("Data too short or too long"),
                            Err(TLSHError::Variety) => {
                                panic!("Data does not have sufficient variety")
                            }
                        };
                    }
                    (_, Input::Digest(ref digest)) => {
                        let h = TLSH::from_digest(digest).unwrap();
                        color = ColorSupplyState::Specified(h.color);
                        *hash = Some(h);
                    }
                    _ => {}
                }
            }
        }
    }

    for hash in hashes.iter() {
        let hash = hash.as_ref().unwrap();
        println!("{}", hash.to_digest());
    }

    if hashes.len() == 2 {
        let diff = TLSH::diff(hashes[0].as_ref().unwrap(), hashes[1].as_ref().unwrap());
        println!("Difference: {diff}");
    }
}
