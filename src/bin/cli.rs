#[cfg(any(not(feature = "dbus"), not(feature = "cli")))]
fn main() {
    panic!("Cli or dbus werent enabled while compiling")
}

#[cfg(all(feature = "dbus", feature = "cli"))]
fn main() {
    todo!()
}
