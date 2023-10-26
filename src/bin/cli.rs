//! Simple dynamic completion example

#[cfg(feature = "cli")]
mod cli {
    use bpaf::*;

    #[derive(Debug, Clone, Bpaf)]
    pub struct Options {
        // Get {}   
    }
}

#[cfg(feature = "cli")]
fn main() {
    // zvariant::Value
    // println!("{:?}", options().run())
}

#[cfg(not(feature = "cli"))]
fn main() {
    panic!("Cli or dbus werent enabled while compiling")
}
