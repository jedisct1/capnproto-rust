#[crate_id="zmq-explorers"];
#[crate_type = "bin"];

extern mod zmq = "rust-zmq";
extern mod capnp;
extern mod extra;

pub mod capnp_zmq;
pub mod explorers_capnp;
pub mod explorer;
pub mod collector;
pub mod viewer;


fn usage(s : &str) {
    error!("usage: {} [explorer|collector|viewer]", s);
    std::os::set_exit_status(1);
}

pub fn main() {

    let args = std::os::args();

    if args.len() < 2 {
        usage(args[0]);
        return;
    }

    match args[1] {
        ~"explorer" => explorer::main(),
        ~"collector" => collector::main(),
        ~"viewer" => viewer::main(),
        _ => usage(args[0]),
    }

}
