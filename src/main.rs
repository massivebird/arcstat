use std::env;
use arcstat::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);

    arcstat::run(config);
}
