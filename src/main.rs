use core::time;
use std::thread::sleep;

use clap::{crate_version, Arg, Command};
use fuse::fuse_impl::mount_fuse;
// use pod::pod::Pod;

mod fuse; // used in pod
// mod pod;
mod providers;

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
    {
        println!("mounting");
        let _session = mount_fuse(mountpoint);
        println!("mounted");
        sleep(time::Duration::from_secs(20));
        println!("unmounting");
    }
    println!("unmounted");
}
