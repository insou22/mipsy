use mipsy_web::worker::MipsyWebWorker;
use wasm_logger::Config;
use gloo_worker::Registrable;

fn main() {
    wasm_logger::init(Config::default());

    MipsyWebWorker::registrar().register();
}
