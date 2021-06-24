use crate::traits::vec_traits::*;
use std::process::Command;

pub fn yarn_install(path: &String) {
    println!("Running yarn install...");
    let output = Command::new("yarn")
        .arg("install")
        .arg("--frozen-lockfile")
        .arg("--cwd")
        .arg(&path.to_string())
        .output()
        .expect("Yarn error!");
    output.stderr.log();
}

pub fn yarn_outdated(path: &String) -> String {
    println!("Running yarn outdated...");
    let output = Command::new("yarn")
        .arg("outdated")
        .arg("--json")
        .arg("--cwd")
        .arg(&path.to_string())
        .output()
        .expect("Yarn error!");
    output.stderr.log();
    let output_json = output.stdout.get_string_or_die();
    return output_json
        .split('\n')
        .skip(1)
        .next()
        .expect("Cannot split yarn outdated json result.")
        .to_string();
}

pub fn yarn_add(path: &String, package: &String, version: &String) {
    println!("Running yarn add {}@^{}...", package, version);
    let output = Command::new("yarn")
        .arg("add")
        .arg(format!("{}@^{}", package, version))
        .arg("--cwd")
        .arg(&path.to_string())
        .output()
        .expect("Yarn error!");
    output.stderr.log();
    output.stdout.log();
}

pub fn yarn_upgrade(path: &String, package: &String, version: &String) {
    println!("Running yarn upgrade {}@^{}...", package, version);
    let output = Command::new("yarn")
        .arg("upgrade")
        .arg(format!("{}@^{}", package, version))
        .arg("--cwd")
        .arg(&path.to_string())
        .output()
        .expect("Yarn error!");
    output.stderr.log();
    output.stdout.log();
}
