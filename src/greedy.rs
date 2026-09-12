use crate::problem::{Input, TreeSearchState};
use crate::util::TimeKeeper;

/// 貪欲法: 各ステップで最良の1手を選ぶ。
///
/// P (Problem) は TreeSearchState を実装した状態型（beam から dedup_key を除いたもの）。
pub fn run<P>(mut cur: P, input: &Input, timer: &TimeKeeper, max_depth: usize, maximize: bool) -> P
where
    P: TreeSearchState,
{
    let mut best = cur.clone();
    let mut best_score = best.score(input);
    let mut actions = Vec::new();

    for depth in 0..max_depth {
        if timer.is_over() || cur.is_terminal(input) {
            break;
        }

        actions.clear();
        cur.enumerate_actions_with_timer(input, Some(timer), &mut actions);
        if actions.is_empty() {
            break;
        }

        let mut chosen: Option<P> = None;
        let mut chosen_score = if maximize { i64::MIN } else { i64::MAX };

        for a in &actions {
            if timer.is_over() {
                break;
            }

            let nxt = cur.apply_action(input, a);
            let sc = nxt.score(input);
            let better = if maximize {
                sc > chosen_score
            } else {
                sc < chosen_score
            };
            if better {
                chosen = Some(nxt);
                chosen_score = sc;
            }
        }

        let Some(next_state) = chosen else {
            break;
        };

        cur = next_state;
        let cur_score = chosen_score;
        let better_than_best = if maximize {
            cur_score > best_score
        } else {
            cur_score < best_score
        };
        if better_than_best {
            best = cur.clone();
            best_score = cur_score;
        }

        crate::trace!(
            "[GREEDY] depth={} score={} time={:.2}",
            depth,
            cur_score,
            timer.progress()
        );
    }

    crate::trace!("[GREEDY] best_score={}", best_score);
    best
}
