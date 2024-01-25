use clap::Parser;
use core::AppConfig;

fn main() {
    let app_config = AppConfig::parse();
    app_config.build(None, None).run();
}
