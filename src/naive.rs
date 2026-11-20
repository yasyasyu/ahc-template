use crate::problem::{Input, NaiveState};
use crate::util::TimeKeeper;

/// 愚直解・直接構築: 汎用探索の枠に乗らない解法を problem 側の solve_naive に任せる。
///
/// P (Problem) は NaiveState を実装した状態型。
/// ここでは呼び出しとログ出力だけを担当し、解法本体は problem.rs に書く。
pub fn run<P>(initial: P, input: &Input, timer: &TimeKeeper) -> P
where
    P: NaiveState,
{
    let best = initial.solve_naive(input, timer);

    crate::trace!(
        "[NAIVE] score={} time={:.2}",
        best.score(input),
        timer.progress()
    );

    best
}
