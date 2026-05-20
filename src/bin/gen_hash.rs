use bcrypt::hash;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin gen_hash <password>");
        std::process::exit(1);
    }
    match hash(&args[1], 10) {
        Ok(h) => println!("{}", h),
        Err(e) => {
            eprintln!("Failed to hash password: {}", e);
            std::process::exit(1);
        }
    }
}
