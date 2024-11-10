use vergen::{vergen, Config};

fn main() {
    if std::env::var("PROFILE").unwrap() == "release" {
        let mut config = Config::default();
        *config.build_mut().timestamp_mut() = true;
        *config.git_mut().sha_mut() = true;
        vergen(config).expect("Unable to generate version information!");
    }
}
