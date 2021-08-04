pub mod models;
pub mod services;
pub mod traits;

use models::structs::*;
use services::actions::*;
use services::n::*;
use structopt::StructOpt;

pub fn main() {
    let args = Params::from_args();
    set_node_version(&args.node_version);
    match &args.action[..] {
        "link" => link(&args.project, &args.path),
        "copy" => copy(&args.project, &args.path),
        "unlink" => unlink(&args.project, &args.path),
        "update" => update(&args.project, &args.path),
        "clean" => clean(&args.project, &args.path),
        "build" => build(&args.project, &args.path),
        _ => println!("Command not found!"),
    }
}
