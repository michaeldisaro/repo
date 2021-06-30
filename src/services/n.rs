use crate::traits::vec_traits::VecExtension;
use std::process::Command;

pub fn set_node_version(version: &String) {
    if version.is_empty() {
        return;
    }
    println!("Setting node version {}...", version);
    let output = Command::new("n").arg(version).output().expect("n error!");
    output.stderr.log_and_die_if_exists();
}
