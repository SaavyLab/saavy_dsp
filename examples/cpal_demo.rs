#[cfg(feature = "cpal-demo")]
fn main() {
    println!("cpal demo stub: enable audio rendering once the engine is implemented.");
}

#[cfg(not(feature = "cpal-demo"))]
fn main() {
    eprintln!("Build with --features cpal-demo to run this example.");
}
