pub mod args;
pub mod cleaner;

use cleaner::Cleaner;

fn main() {
    match Cleaner::init().start() {
        Ok(result) => println!("{}", result),
        Err(err) => println!("{:?}", err),
    }
}
