#![expect(rustdoc::missing_crate_level_docs)] // it's an example

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
mod app;

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
fn main() -> std::io::Result<()> {
    app::run()
}

// Do not check `app` on unsupported platforms when check "--all-features" is used in CI.
#[cfg(not(all(target_os = "linux", not(target_env = "ohos"))))]
fn main() {
    #![expect(clippy::print_stdout)]
    println!("This example only supports Linux.");
}
