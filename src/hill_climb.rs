use rand::Rng;

use crate::problem::{Input, LocalSearchState};
use crate::util::TimeKeeper;

/// 山登り法: annealing と同じ LocalSearchState を使うが、改善遷移のみ受理する
/// （温度パラメータを持たない分だけ annealing::run より単純）。
pub fn run<P, R>(mut cur: P, input: &Input, rng: &mut R, timer: &TimeKeeper, maximize: bool) -> P
where
    P: LocalSearchState,
    R: Rng + ?Sized,
{
    let mut cur_score = cur.score(input);
    let mut best = cur.clone();
    let mut best_score = cur_score;
    let mut iter = 0usize;
    let mut accepted = 0usize;

    while !timer.is_over() {
        iter += 1;
        let Some(nb) = cur.neighbor(input, rng) else {
            continue;
        };

        cur.apply_neighbor(input, &nb);
        let nxt_score = cur.score(input);
        let improved = if maximize {
            nxt_score >= cur_score
        } else {
            nxt_score <= cur_score
        };

        if improved {
            accepted += 1;
            cur_score = nxt_score;
            let best_improved = if maximize {
                cur_score > best_score
            } else {
                cur_score < best_score
            };
            if best_improved {
                best = cur.clone();
                best_score = cur_score;
            }
        } else {
            cur.rollback_neighbor(input, &nb);
        }
    }

    crate::trace!(
        "[HC] iter={} accepted={} best_score={}",
        iter,
        accepted,
        best_score
    );

    best
}
