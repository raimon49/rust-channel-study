use std::thread::spawn;
use std::sync::mpsc::channel;
use std::rc::Rc;

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

    let rc1 = Rc::new("hello threads".to_string());
    let rc2 = rc1.clone();
    spawn(move || {
//  ^^^^^ `std::rc::Rc<std::string::String>` cannot be sent between threads safely
//         以下の行が有効になっているとRustコンパイラは上記のエラーを発生させる
//         rc2.clone();
    });
    rc1.clone();

    use std::sync::mpsc;
    pub trait OffThreadExt: Iterator {
        fn off_thread(self) -> mpsc::IntoIter<Self::Item>;
    }

    impl<T> OffThreadExt for T
        where T: Iterator + Send + 'static, // 型Tのイテレータをspawn()で新しいスレッドに移動するため Iterator + Send + 'staticの宣言が必要
              T::Item: Send + 'static       // アイテムをchannel()経由で送信するためSend + 'staticの宣言が必要
    {
        fn off_thread(self) -> mpsc::IntoIter<Self::Item> {
            let (sender, receiver) = mpsc::sync_channel(1024);
            spawn(move || {
                for item in self {
                    if sender.send(item).is_err() {
                        break;
                    }
                }
            });

            receiver.into_iter()
        }
    }
}
