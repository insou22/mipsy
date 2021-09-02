extern crate vergen;

use vergen::{ConstantsFlags, generate_cargo_keys};

fn main() {
    generate_cargo_keys(ConstantsFlags::SHA_SHORT | ConstantsFlags::COMMIT_DATE)
            .expect("Unable to generate the cargo keys!");
}
