mod client;
mod server;

static BIND_ADDRESS: &str = "127.0.0.1:8888";

fn main() -> Result<(), &'static str> {
    let mut args = std::env::args();

    match args.nth(1).as_ref().map(String::as_ref) {
        Some("client") => client::main(BIND_ADDRESS),
        Some("server") => server::main(BIND_ADDRESS),
        _ => return Err("Usage: cargo run (client|server)"),
    }

    Ok(())
}
