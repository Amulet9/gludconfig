//! Simple dynamic completion example

#[cfg(feature = "cli")]
mod cli {
    use bpaf::*;

    #[derive(Debug, Clone, Bpaf)]
    pub struct GludCli {}
}

#[cfg(feature = "cli")]
fn main() {
    // println!("{:?}", options().run())
}

#[cfg(not(feature = "cli"))]
fn main() {
    panic!("Cli or dbus werent enabled while compiling")
}
