use rand::Rng;

use crate::problem::{Input, LocalSearchState};
use crate::util::TimeKeeper;

/// 焼きなまし法: 時間いっぱいまで近傍遷移を試す。
///
/// P (Problem) は LocalSearchState を実装した状態型。
/// アルゴリズムはこのトレイト越しにしか State を触らないため、
/// 「焼きなましが要求する役割」が LocalSearchState の定義を読むだけで分かる。
pub fn run<P, R>(
    mut cur: P,
    input: &Input,
    rng: &mut R,
    timer: &TimeKeeper,
    start_temp: f64,
    end_temp: f64,
    maximize: bool,
) -> P
where
    P: LocalSearchState,
    R: Rng + ?Sized,
{
    let mut cur_score = cur.score(input);
    let mut best = cur.clone();
    let mut best_score = cur_score;
    let mut iter = 0usize;
    let mut accepted = 0usize;
    let temp_ratio_ln = (end_temp / start_temp).ln();

    loop {
        let elapsed_sec = timer.elapsed_sec();
        if timer.is_over_elapsed(elapsed_sec) {
            break;
        }

        iter += 1;
        let Some(nb) = cur.neighbor(input, rng) else {
            continue;
        };

        let progress = timer.progress_from_elapsed(elapsed_sec);
        let t = start_temp * (temp_ratio_ln * progress).exp().max(1e-12);
        cur.apply_neighbor(input, &nb);
        let nxt_score = cur.score(input);
        let diff = if maximize {
            (nxt_score - cur_score) as f64
        } else {
            (cur_score - nxt_score) as f64
        };

        if diff >= 0.0 || rng.random::<f64>() < (diff / t).exp() {
            accepted += 1;
            cur_score = nxt_score;
            let improved = if maximize {
                cur_score > best_score
            } else {
                cur_score < best_score
            };
            if improved {
                best = cur.clone();
                best_score = cur_score;
            }
        } else {
            cur.rollback_neighbor(input, &nb);
        }
    }

    crate::trace!(
        "[SA] iter={} accepted={} best_score={}",
        iter,
        accepted,
        best_score
    );

    best
}
