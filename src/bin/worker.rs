use mipsy_web::worker::Worker;
use wasm_logger::Config;
use yew_agent::Threaded;
fn main() {
    wasm_logger::init(Config::default());
    Worker::register();
}
