#![allow(dead_code, non_snake_case, unused_macros)]

const TIME_LIMIT_SEC: f64 = 1.95;
const START_TEMP: f64 = 1e4;
const END_TEMP: f64 = 1e1;
const BEAM_WIDTH: usize = 200;
const BEAM_DEPTH: usize = 60;
const GREEDY_DEPTH: usize = 1_000_000;
// score が大きいほど良いなら true、小さいほど良いなら false。
const MAXIMIZE: bool = true;

macro_rules! trace {
    ($($arg:tt)*) => {
        if cfg!(feature = "local") {
            eprintln!($($arg)*);
        }
    };
}

const SOLVER_TYPE: SolverType = SolverType::SA;

#[derive(Clone, Copy)]
enum SolverType {
    SA,
    HillClimb,
    Greedy,
    Beam,
}

fn main() {
    let input = problem::Input::read();
    let best: problem::State = solve(&input);
    problem::output(&best, &input);
}

fn solve(input: &problem::Input) -> problem::State {
    // 問題依存の初期解は problem::State::new 側で構築する。
    let initial = problem::State::new(input);

    match SOLVER_TYPE {
        SolverType::SA => {
            // 焼きなまし: 時間いっぱいまで近傍遷移を試す。
            let timer = util::TimeKeeper::new(TIME_LIMIT_SEC);
            let mut rng = util::rng(1);
            annealing::run(
                initial,
                &mut rng,
                &timer,
                START_TEMP,
                END_TEMP,
                MAXIMIZE,
                |s| s.score(input),
                |s, rng| s.neighbor(input, rng),
                |s, nb| s.apply_neighbor(input, nb),
                |s, nb| s.rollback_neighbor(input, nb),
            )
        }
        SolverType::HillClimb => {
            // 山登り: 改善遷移のみ受理する。
            let timer = util::TimeKeeper::new(TIME_LIMIT_SEC);
            let mut rng = util::rng(1);
            hill_climb::run(
                initial,
                &mut rng,
                &timer,
                MAXIMIZE,
                |s| s.score(input),
                |s, rng| s.neighbor(input, rng),
                |s, nb| s.apply_neighbor(input, nb),
                |s, nb| s.rollback_neighbor(input, nb),
            )
        }
        SolverType::Greedy => {
            // 貪欲: 各ステップで最良の1手を選ぶ。
            greedy::run(
                initial,
                GREEDY_DEPTH,
                MAXIMIZE,
                |s| s.score(input),
                |s| s.is_terminal(input),
                |s, dst| s.enumerate_actions(input, dst),
                |s, a| s.apply_action(input, a),
            )
        }
        // ビームサーチ: 深さと幅を固定して段階的に展開する。
        SolverType::Beam => beam::run(
            initial,
            BEAM_WIDTH,
            BEAM_DEPTH,
            |s| s.score(input),
            |s| s.is_terminal(input),
            |s, dst| s.enumerate_actions(input, dst),
            |s, a| s.apply_action(input, a),
            |s| s.dedup_key(input),
        ),
    }
}

mod problem {
    use proconio::*;
    use rand::Rng;

    pub struct Input;

    impl Input {
        /// 入力を読み取り、問題インスタンスを構築する。
        ///
        /// ここでは盤面・グラフ・制約など、探索で参照する不変データを保持する。
        pub fn read() -> Self {
            input! {}
            Self
        }
    }

    #[derive(Clone, Debug)]
    pub struct Neighbor;

    #[derive(Clone, Debug)]
    pub struct State;

    impl State {
        /// 初期解を構築する。
        ///
        /// 貪欲・乱択・固定初期値など、探索の開始状態をここで作る。
        pub fn new(_input: &Input) -> Self {
            todo!()
        }

        /// 状態の評価値（目的関数）を返す。
        ///
        /// 大きいほど良い設計なら maximize=true のまま使える。
        /// 小さいほど良い評価関数にする場合は、グローバル定数 MAXIMIZE を false にする。
        pub fn score(&self, _input: &Input) -> i64 {
            todo!()
        }

        /// 現在状態から有効な近傍操作を1つ生成する（焼きなまし用）。
        ///
        /// 有効な遷移がない場合は None を返す。
        pub fn neighbor<R: Rng + ?Sized>(&self, _input: &Input, _rng: &mut R) -> Option<Neighbor> {
            todo!()
        }

        /// 近傍操作を状態へ適用する（焼きなまし用）。
        pub fn apply_neighbor(&mut self, _input: &Input, _nb: &Neighbor) {
            todo!()
        }

        /// 直前に適用した近傍操作を巻き戻す（焼きなまし用）。
        ///
        /// apply_neighbor と対になる逆操作を実装する。
        pub fn rollback_neighbor(&mut self, _input: &Input, _nb: &Neighbor) {
            todo!()
        }

        /// 1手先の候補手を列挙する（ビームサーチ用）。
        ///
        /// dst は使い回されるため、先頭で clear してから push するのが安全。
        /// Greedy でもこの関数を使う。
        pub fn enumerate_actions(&self, _input: &Input, _dst: &mut Vec<Neighbor>) {
            todo!()
        }

        /// 候補手を適用した次状態を返す（ビームサーチ用）。
        pub fn apply_action(&self, _input: &Input, _action: &Neighbor) -> Self {
            todo!()
        }

        /// ビーム展開を打ち切る終端条件を返す。
        ///
        /// 例: 手数上限到達、全制約充足、これ以上遷移不能。
        pub fn is_terminal(&self, _input: &Input) -> bool {
            todo!()
        }

        /// 重複状態除去に使うキーを返す（ビームサーチ用）。
        ///
        /// 同一とみなす状態が同じキーになるように設計する。
        pub fn dedup_key(&self, _input: &Input) -> u64 {
            todo!()
        }
    }

    /// 最終解を出力する。
    ///
    /// ジャッジ仕様に合わせて、State から提出形式の文字列を生成する。
    pub fn output(_state: &State, _input: &Input) {
        println!();
    }
}

mod annealing {
    use rand::Rng;

    use crate::util::TimeKeeper;

    pub fn run<S, N, R, FScore, FNeighbor, FApply, FRollback>(
        mut cur: S,
        rng: &mut R,
        timer: &TimeKeeper,
        start_temp: f64,
        end_temp: f64,
        maximize: bool,
        mut score: FScore,
        mut neighbor: FNeighbor,
        mut apply: FApply,
        mut rollback: FRollback,
    ) -> S
    where
        S: Clone,
        N: Clone,
        R: Rng + ?Sized,
        FScore: FnMut(&S) -> i64,
        FNeighbor: FnMut(&S, &mut R) -> Option<N>,
        FApply: FnMut(&mut S, &N),
        FRollback: FnMut(&mut S, &N),
    {
        let mut cur_score = score(&cur);
        let mut best = cur.clone();
        let mut best_score = cur_score;
        let mut iter = 0usize;
        let mut accepted = 0usize;

        while !timer.is_over() {
            iter += 1;
            let Some(nb) = neighbor(&cur, rng) else {
                continue;
            };

            let t = start_temp * (end_temp / start_temp).powf(timer.progress()).max(1e-12);
            apply(&mut cur, &nb);
            let nxt_score = score(&cur);
            let diff = if maximize {
                (nxt_score - cur_score) as f64
            } else {
                (cur_score - nxt_score) as f64
            };

            if diff >= 0.0 || rng.gen::<f64>() < (diff / t).exp() {
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
                rollback(&mut cur, &nb);
            }
        }

        trace!(
            "[SA] iter={} accepted={} best_score={}",
            iter,
            accepted,
            best_score
        );

        best
    }
}

mod beam {
    use rustc_hash::FxHashSet;

    pub fn run<S, A, FScore, FTerminal, FEnum, FApply, FKey>(
        initial: S,
        width: usize,
        depth: usize,
        mut score: FScore,
        mut is_terminal: FTerminal,
        mut enumerate_actions: FEnum,
        mut apply_action: FApply,
        mut dedup_key: FKey,
    ) -> S
    where
        S: Clone,
        A: Clone,
        FScore: FnMut(&S) -> i64,
        FTerminal: FnMut(&S) -> bool,
        FEnum: FnMut(&S, &mut Vec<A>),
        FApply: FnMut(&S, &A) -> S,
        FKey: FnMut(&S) -> u64,
    {
        let mut best = initial.clone();
        let mut best_score = score(&best);
        let mut cur = vec![initial];

        for d in 0..depth {
            let cur_width = cur.len();
            let mut nxt = Vec::new();
            for st in &cur {
                if is_terminal(st) {
                    nxt.push(st.clone());
                    continue;
                }
                let mut actions = Vec::new();
                enumerate_actions(st, &mut actions);
                for a in actions {
                    nxt.push(apply_action(st, &a));
                }
            }

            if nxt.is_empty() {
                break;
            }

            nxt.sort_by_key(|s| std::cmp::Reverse(score(s)));
            let mut seen = FxHashSet::default();
            nxt.retain(|s| seen.insert(dedup_key(s)));
            if nxt.len() > width {
                nxt.truncate(width);
            }

            trace!(
                "[BEAM] depth={} cur_width={} next_width={}",
                d,
                cur_width,
                nxt.len(),
            );

            if let Some(top) = nxt.first() {
                let sc = score(top);
                if sc > best_score {
                    best = top.clone();
                    best_score = sc;
                }
            }
            cur = nxt;
        }

        trace!("[BEAM] best_score={}", best_score);

        best
    }
}

mod hill_climb {
    use rand::Rng;

    use crate::util::TimeKeeper;

    pub fn run<S, N, R, FScore, FNeighbor, FApply, FRollback>(
        mut cur: S,
        rng: &mut R,
        timer: &TimeKeeper,
        maximize: bool,
        mut score: FScore,
        mut neighbor: FNeighbor,
        mut apply: FApply,
        mut rollback: FRollback,
    ) -> S
    where
        S: Clone,
        N: Clone,
        R: Rng + ?Sized,
        FScore: FnMut(&S) -> i64,
        FNeighbor: FnMut(&S, &mut R) -> Option<N>,
        FApply: FnMut(&mut S, &N),
        FRollback: FnMut(&mut S, &N),
    {
        let mut cur_score = score(&cur);
        let mut best = cur.clone();
        let mut best_score = cur_score;
        let mut iter = 0usize;
        let mut accepted = 0usize;

        while !timer.is_over() {
            iter += 1;
            let Some(nb) = neighbor(&cur, rng) else {
                continue;
            };

            apply(&mut cur, &nb);
            let nxt_score = score(&cur);
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
                rollback(&mut cur, &nb);
            }
        }

        trace!(
            "[HC] iter={} accepted={} best_score={}",
            iter,
            accepted,
            best_score
        );

        best
    }
}

mod greedy {
    pub fn run<S, A, FScore, FTerminal, FEnum, FApply>(
        mut cur: S,
        max_depth: usize,
        maximize: bool,
        mut score: FScore,
        mut is_terminal: FTerminal,
        mut enumerate_actions: FEnum,
        mut apply_action: FApply,
    ) -> S
    where
        S: Clone,
        A: Clone,
        FScore: FnMut(&S) -> i64,
        FTerminal: FnMut(&S) -> bool,
        FEnum: FnMut(&S, &mut Vec<A>),
        FApply: FnMut(&S, &A) -> S,
    {
        let mut best = cur.clone();
        let mut best_score = score(&best);

        for depth in 0..max_depth {
            if is_terminal(&cur) {
                break;
            }

            let mut actions = Vec::new();
            enumerate_actions(&cur, &mut actions);
            if actions.is_empty() {
                break;
            }

            let mut chosen: Option<S> = None;
            let mut chosen_score = if maximize { i64::MIN } else { i64::MAX };

            for a in &actions {
                let nxt = apply_action(&cur, a);
                let sc = score(&nxt);
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
            let cur_score = score(&cur);
            let better_than_best = if maximize {
                cur_score > best_score
            } else {
                cur_score < best_score
            };
            if better_than_best {
                best = cur.clone();
                best_score = cur_score;
            }

            trace!("[GREEDY] depth={} score={}", depth, cur_score);
        }

        trace!("[GREEDY] best_score={}", best_score);
        best
    }
}

mod util {
    use rand_pcg::Pcg64Mcg;
    use std::time::Instant;

    pub struct TimeKeeper {
        start: Instant,
        limit_sec: f64,
    }

    impl TimeKeeper {
        pub fn new(limit_sec: f64) -> Self {
            Self {
                start: Instant::now(),
                limit_sec,
            }
        }

        pub fn is_over(&self) -> bool {
            self.start.elapsed().as_secs_f64() >= self.limit_sec
        }

        pub fn progress(&self) -> f64 {
            (self.start.elapsed().as_secs_f64() / self.limit_sec).clamp(0.0, 1.0)
        }
    }

    pub fn rng(seed: u64) -> Pcg64Mcg {
        Pcg64Mcg::new(seed as u128)
    }
}
