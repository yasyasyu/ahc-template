//! 各探索アルゴリズムが State に要求する役割(トレイト)の定義。
//!
//! 実装は src/problem.rs 側で行う。ここはアルゴリズムとの契約なので基本的に触らない。

use crate::engine::util::TimeKeeper;
use crate::problem::Input;
use rand::Rng;

/// 状態の評価値（目的関数）を持つことを表す、全探索アルゴリズム共通の役割。
///
/// 大きいほど良い設計なら maximize=true のまま使える。
/// 小さいほど良い評価関数にする場合は、main.rs の DEFAULT_MAXIMIZE (または params.toml の maximize) を false にする。
pub trait Scored {
    fn score(&self, input: &Input) -> i64;
}

/// 焼きなまし法(annealing)・山登り法(hill_climb)が要求する役割。
///
/// 「現在の状態を少しだけ動かして、ダメなら元に戻す」という
/// 差分更新スタイルの探索に必要な操作をまとめたもの。
pub trait LocalSearchState: Scored + Clone {
    type Neighbor: Clone;

    /// 現在状態から有効な近傍操作を1つ生成する。
    ///
    /// 有効な遷移がない場合は None を返す。
    fn neighbor<R: Rng + ?Sized>(&self, input: &Input, rng: &mut R) -> Option<Self::Neighbor>;

    /// 近傍操作を状態へ適用する。
    fn apply_neighbor(&mut self, input: &Input, nb: &Self::Neighbor);

    /// 直前に適用した近傍操作を巻き戻す。
    ///
    /// apply_neighbor と対になる逆操作を実装する。
    fn rollback_neighbor(&mut self, input: &Input, nb: &Self::Neighbor);
}

/// 貪欲法(greedy)・ビームサーチ(beam)が要求する役割。
///
/// 「現在の状態から次状態候補を複数列挙し、良いものを残す」という
/// 木探索スタイルの探索に必要な操作をまとめたもの。
pub trait TreeSearchState: Scored + Clone {
    type Action: Clone;

    /// 展開を打ち切る終端条件を返す。
    ///
    /// 例: 手数上限到達、全制約充足、これ以上遷移不能。
    fn is_terminal(&self, input: &Input) -> bool;

    /// 1手先の候補手を列挙する。
    ///
    /// dst は使い回されるため、先頭で clear してから push するのが安全。
    fn enumerate_actions(&self, input: &Input, dst: &mut Vec<Self::Action>);

    /// 時間に応じて候補列挙を打ち切りたい場合の拡張フック。
    ///
    /// デフォルトでは enumerate_actions にそのまま委譲する。
    fn enumerate_actions_with_timer(
        &self,
        input: &Input,
        _timer: Option<&TimeKeeper>,
        dst: &mut Vec<Self::Action>,
    ) {
        self.enumerate_actions(input, dst);
    }

    /// 候補手を適用した次状態を返す。
    fn apply_action(&self, input: &Input, action: &Self::Action) -> Self;
}

/// ビームサーチ(beam)のみが追加で要求する役割（重複状態の除去）。
pub trait BeamSearchState: TreeSearchState {
    /// 重複状態除去に使うキーを返す。
    ///
    /// 同一とみなす状態が同じキーになるように設計する。
    fn dedup_key(&self, input: &Input) -> u64;
}

/// 愚直解・全探索・手書きの構築解など、汎用探索の枠に収まらない解法の役割(naive)。
///
/// 近傍や候補手の形に落とし込まず、Input から直接解を作りたいときに使う。
/// - 提出前の動作確認用に、まず正しい出力を出す最小解法を書く
/// - 小さいケースで全探索して、他ソルバのスコアと突き合わせる
/// - 問題固有の構築アルゴリズムをそのまま書く
pub trait NaiveState: Scored + Sized {
    /// initial (State::new の結果) を起点に解を構築して返す。
    ///
    /// initial を使わない解法なら無視して一から作ってよい。
    /// 時間制限を気にする場合は timer を参照する。
    fn solve_naive(self, input: &Input, timer: &TimeKeeper) -> Self;
}
