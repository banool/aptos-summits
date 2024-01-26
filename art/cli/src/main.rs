use artcore::AppConfig;
use clap::Parser;

fn main() {
    let app_config = AppConfig::parse();
    app_config.build(None).run();
}
