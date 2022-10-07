extern crate vergen;

use vergen::{Config, vergen, ShaKind, TimestampKind};


fn main() {
    let mut config = Config::default();

    *config.git_mut().enabled_mut() = true;

    *config.git_mut().sha_mut() = true;
    *config.git_mut().sha_kind_mut() = ShaKind::Short;

    *config.git_mut().commit_timestamp_mut() = true;
    *config.git_mut().commit_timestamp_kind_mut() = TimestampKind::DateOnly;

    vergen(config).expect("Unable to generate the cargo keys!");
}
