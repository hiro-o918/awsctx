use crate::ctx;
use ansi_term::Colour::Green;

pub fn show_contexts(contexts: &[ctx::Context]) {
    for c in contexts.iter() {
        if c.active {
            println!("{}", Green.bold().paint(format!("* {}", c.name)))
        } else {
            println!("  {}", c.name);
        }
    }
}

pub fn show_active_context(contexts: &[ctx::Context]) {
    for c in contexts.iter() {
        if c.active {
            println!("{}", c.name)
        }
    }
}
