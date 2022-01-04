use std::thread::spawn;
use std::sync::mpsc::channel;

fn main() {
    let (sender, receiver) = channel();

    let handle = spawn(move || {
        let mut text = String::new();

        if sender.send(text).is_err() {
            return;
        }

        // Ok(())
    });
}
