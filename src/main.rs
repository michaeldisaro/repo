pub mod models;
pub mod services;
pub mod traits;

use models::structs::*;
use services::actions::*;
use structopt::StructOpt;

fn main() {
    let args = Params::from_args();
    match &args.action[..] {
        "link" => link(&args.project, &args.path),
        "copy" => copy(&args.project, &args.path),
        "unlink" => unlink(&args.project, &args.path),
        "update" => update(&args.project, &args.path),
        _ => println!("Command not found!"),
    }
}
