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
    {
        // プレイヤーリストを管理するゲームサーバーの仮想コード
        use std::sync::Arc;
        use std::sync::Mutex;
        use std::sync::RwLock;
        use std::io;

        // プレイヤーは固有のIDを持つ
        type PlayerId = u32;
        // 待ちプレイヤーリストはGAME_SIZEよりも長くならない
        const GAME_SIZE: usize = 8;
        // 待ちプレイヤーリストはコレクションとして実装する
        type WaitingList = Vec<PlayerId>;

        struct AppConfig {
            num: u32
        }

        struct FermEmpireApp {
            config: RwLock<AppConfig>,
            waiting_list: Mutex<WaitingList>
        }

        impl FermEmpireApp {
            fn join_waiting_list(&self, player: PlayerId) {
                // Mutexで囲まれたデータにアクセスするにはlock()を呼んで排他ロックを取得
                // 返される値はMutexGuard<WaitingList>型
                let mut guard = self.waiting_list.lock().unwrap();
                // MutexGuard<WaitingList>は、&mut WaitingListを薄いラッパーでくるんだ型のため、WaitingListのメソッドを直接呼ぶことが可能
                // Vec<PlayerId>のpush()は&mut selfを要求するが、Mutexは排他を保証するためコンパイルできる
                guard.push(player);

                // 保有済みのロックをもう一度取得しようとするとデッドロックになる
                // let mut duplicated_lock = self.waiting_list.lock().unwrap();

                if guard.len() == GAME_SIZE {
                    let players = guard.split_off(0);
                    // 取得した先頭の待ちプレイヤーにゲームを開始させる
                    // self.start_game(players);
                }
            }

            fn read_config(&self) {
                // リードロックを取得
                let config_guard = self.config.read().unwrap();
                println!("{}", config_guard.num);
            }

            fn reload_config(&self) -> io::Result<()> {
                let new_config = AppConfig{ num: 100 };
                // ライトロックを取得
                let mut config_guard = self.config.write().unwrap();
                *config_guard = new_config;
                Ok(())
            }
        }

        // サーバー起動時に待ちプレイヤーを持つオブジェクトをArcで囲ったシングルトンとして作成
        let app = Arc::new(FermEmpireApp {
            config: RwLock::new(AppConfig{ num: 1 }),
            waiting_list: Mutex::new(vec![])
        });
    }
    {
        pub mod shared_channel {
            use std::sync::{Arc, Mutex};
            use std::sync::mpsc::{channel, Sender, Receiver};

            // Arc（ヒープ確保・アトミック参照カウント）
            // Mutex（排他保護）
            // Receiver（レシーバ値）
            // T（ジェネリクスでラップしたい型）
            #[derive(Clone)]
            pub struct SharedReceiver<T>(Arc<Mutex<Receiver<T>>>);

            impl<T> Iterator for SharedReceiver<T> {
                type Item = T;

                fn next(&mut self) -> Option<T> {
                    let guard = self.0.lock().unwrap();
                    guard.recv().ok()
                }
            }

            // Receiverを複数のスレッドで共有できる新しいchannelを作る
            // stdlibのchannel()をこのラッパーで置き換えることも可能
            pub fn shared_channel<T>() -> (Sender<T>, SharedReceiver<T>) {
                let (sender, receiver) = channel();
                (sender, SharedReceiver(Arc::new(Mutex::new(receiver))))
            }
        }
    }
    {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

        let atom: AtomicUsize = AtomicUsize::new(1); // 複数スレッドで共有できるアトミックな型のusize
        atom.fetch_add(1, Ordering::SeqCst);         // アトミックな型のインクリメント（SeqCstはメモリオーダー）

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let worker_cancel_flag = cancel_flag.clone();

        // 同じ値を見ているスマートポインタを1つはメインスレッドに留め1つはワーカースレッドに移動する
        let worker_handle = spawn(move || {
            // キャンセルされているかチェック
            if (worker_cancel_flag.load(Ordering::SeqCst)) {
                return None;
            }

            Some(0)
        });

        // メインスレッド側からフラグを真にしてワーカースレッドの処理をキャンセル
        cancel_flag.store(true, Ordering::SeqCst);

        // キャンセルしたワーカースレッドの結果は捨てる（結果はNone）
        worker_handle.join().unwrap();
    }
    {
        static PACKETS_SERVED: usize = 0;
    }
}
