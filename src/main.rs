// The protocol module also defines messages we haven't connected yet.
#[allow(dead_code)]
mod dap_protocol;
mod error;
mod lua_process;
// Some Lua message declarations are retained for future handlers.
#[allow(dead_code)]
mod lua_protocol;
mod session;
mod transport;

fn main() {
    if let Err(error) = session::run() {
        eprintln!("moonbridge: {error}");
        std::process::exit(1);
    }
}
