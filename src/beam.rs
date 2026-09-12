use crate::problem::{BeamSearchState, Input};
use crate::util::TimeKeeper;
use rustc_hash::FxHashMap;

/// ビームサーチ: 深さと幅を固定して段階的に展開する。
///
/// P (Problem) は BeamSearchState (= TreeSearchState + dedup_key) を実装した状態型。
pub fn run<P>(
    initial: P,
    input: &Input,
    timer: &TimeKeeper,
    width: usize,
    depth: usize,
    maximize: bool,
) -> P
where
    P: BeamSearchState,
{
    let mut best = initial.clone();
    let mut best_score = best.score(input);
    let mut cur = vec![initial];
    let mut next_states: Vec<(P, i64)> = Vec::new();
    let mut actions = Vec::new();
    let mut best_by_key: FxHashMap<u64, (P, i64)> = FxHashMap::default();

    let is_better = |lhs: i64, rhs: i64| {
        if maximize {
            lhs > rhs
        } else {
            lhs < rhs
        }
    };

    for d in 0..depth {
        if timer.is_over() {
            break;
        }

        let cur_width = cur.len();
        best_by_key.clear();

        for st in &cur {
            if timer.is_over() {
                break;
            }

            if st.is_terminal(input) {
                let sc = st.score(input);
                let key = st.dedup_key(input);
                if let Some((kept_state, kept_score)) = best_by_key.get_mut(&key) {
                    if is_better(sc, *kept_score) {
                        *kept_state = st.clone();
                        *kept_score = sc;
                    }
                } else {
                    best_by_key.insert(key, (st.clone(), sc));
                }
                continue;
            }

            actions.clear();
            st.enumerate_actions_with_timer(input, Some(timer), &mut actions);
            for a in &actions {
                if timer.is_over() {
                    break;
                }

                let nxt = st.apply_action(input, a);
                let sc = nxt.score(input);
                let key = nxt.dedup_key(input);
                if let Some((kept_state, kept_score)) = best_by_key.get_mut(&key) {
                    if is_better(sc, *kept_score) {
                        *kept_state = nxt;
                        *kept_score = sc;
                    }
                } else {
                    best_by_key.insert(key, (nxt, sc));
                }
            }
        }

        if best_by_key.is_empty() {
            break;
        }

        next_states.clear();
        next_states.reserve(best_by_key.len());
        for (_, (st, sc)) in best_by_key.drain() {
            next_states.push((st, sc));
        }

        if width == 0 {
            cur.clear();
            break;
        }

        if next_states.len() > width {
            next_states.select_nth_unstable_by(width - 1, |a, b| {
                if maximize {
                    b.1.cmp(&a.1)
                } else {
                    a.1.cmp(&b.1)
                }
            });
            next_states.truncate(width);
        }

        let mut layer_best_score = if maximize { i64::MIN } else { i64::MAX };
        let mut layer_best_index: Option<usize> = None;
        for (i, (_, sc)) in next_states.iter().enumerate() {
            if is_better(*sc, layer_best_score) {
                layer_best_score = *sc;
                layer_best_index = Some(i);
            }
        }

        crate::trace!(
            "[BEAM] depth={} cur_width={} next_width={}",
            d,
            cur_width,
            next_states.len(),
        );

        if let Some(top_i) = layer_best_index {
            if is_better(layer_best_score, best_score) {
                best = next_states[top_i].0.clone();
                best_score = layer_best_score;
            }
        }

        cur.clear();
        cur.reserve(next_states.len());
        for (st, _) in next_states.drain(..) {
            cur.push(st);
        }
    }

    crate::trace!("[BEAM] best_score={}", best_score);

    best
}
