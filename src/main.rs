mod services;
mod models;

fn main() {
    println!("Hello, world!");
    println!("{}", std::env::current_exe().unwrap().as_path().to_str().unwrap());
}
