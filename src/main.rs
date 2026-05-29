mod build_system;
mod msg;

fn main() {
    match build_system::entry_point() {
        Ok(_) => (),
        Err(e) => err!(e),
    }
}
