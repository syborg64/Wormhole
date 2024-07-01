use clap::{crate_version, Arg, Command};

mod fuse;

fn main() {
    let matches = Command::new("hello")
        .version(crate_version!())
        .arg(
            Arg::new("MOUNT_POINT")
                .required(true)
                .index(1)
                .help("Act as a client, and mount FUSE at given path"),
        )
        .get_matches();
    let mountpoint = matches.get_one::<String>("MOUNT_POINT").unwrap();
    fuse::start::mount_fuse(mountpoint);
}