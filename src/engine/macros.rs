//! テンプレート全体で使うマクロ。
//!
//! main.rs で `#[macro_use] mod engine;` を problem より先に宣言しているので、
//! problem.rs からも `trace!` / `static_data!` をそのまま呼べる。

/// `--features local` のときだけ標準エラーに出力する。
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        if cfg!(feature = "local") {
            eprintln!($($arg)*);
        }
    };
}

/// `OnceLock` ベースの前計算キャッシュを関数として定義する。
#[macro_export]
macro_rules! static_data {
    ($name:ident:$ty:ty|$expr:expr) => {
        fn $name() -> &'static $ty {
            use std::sync::OnceLock;
            static DATA: OnceLock<$ty> = OnceLock::new();
            DATA.get_or_init(|| $expr)
        }
    };
}
