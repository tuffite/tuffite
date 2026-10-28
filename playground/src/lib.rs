//! Native embedding for the Playground application and its example services.
mod application;
mod browser;
mod logging;
#[allow(non_snake_case)]
mod shell_api;
mod utilities;

#[cfg(test)]
mod codec_tests;

#[tuffite::main]
fn main() -> tuffite::Embedding<application::PlaygroundApplication> {
    tuffite::Embedding::new()
        .browser::<browser::PlaygroundBrowser>()
        .utility::<utilities::EchoUtility>()
        .v8_utility::<utilities::Parser>()
        .utility::<utilities::Worker>()
}
