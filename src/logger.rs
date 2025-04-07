use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

pub fn start_log() {
    let mut builder = Builder::new();
    builder.format(|buf, record| {
        writeln!(buf, "{}: {}: {}", buf.timestamp(), record.level(), record.args())
    }).filter_level(LevelFilter::Info).init();
}
