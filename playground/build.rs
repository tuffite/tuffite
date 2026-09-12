use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set"));
    let framework = tuffite::build::framework::Resolver::from_env()
        .expect("configure Tuffite Framework resolver");
    let shell_api = tuffite::build::shell_api::Generator::new()
        .rust(&out_dir, tuffite::build::shell_api::OutputMode::Write)
        .javascript("generated", tuffite::build::shell_api::OutputMode::Write);
    tuffite::build::Builder::new()
        .add(framework)
        .add(shell_api)
        .build()
        .expect("prepare Tuffite application build");
}
