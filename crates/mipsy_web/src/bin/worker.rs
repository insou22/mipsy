use mipsy_web::worker::MipsyWebWorker;
use wasm_logger::Config;
use yew_agent::PublicWorker;

fn main() {
    wasm_logger::init(Config::default());
    MipsyWebWorker::register();
}
