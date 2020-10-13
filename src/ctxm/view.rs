use core::fmt::Display;

pub fn show_contexts<I: IntoIterator<Item = T>, T: Display>(contexts: I) {
    for c in contexts.into_iter() {
        println!("{}", c);
    }
}
