use std::thread::spawn;
use std::sync::mpsc::channel;

fn main() {
    // mpscはmulti-producer, single-consumer（複数の生産者、単一の消費者）を意味する
    // std::sync::mpsc::sync_channelを使うと同期チャネルになる
    let (sender, receiver) = channel();

    let handle = spawn(move || {
        let mut text = String::new();

        if sender.send(text).is_err() {
            return;
        }

        // Ok(())
    });
}
