//! テンプレートの共通基盤。問題ごとに触る必要はない。
//!
//! 問題依存のコードは src/problem.rs に、調整用の定数は src/main.rs に書く。
//! 提出時に不要なアルゴリズムだけを削って1ファイルにまとめたい場合は tools/bundle.ps1 を使う。
//! (mod 宣言・solve() 内の対応する分岐は `// BUNDLE:<KEY>-BEGIN`〜`-END` で囲ってある)

#[macro_use]
pub mod macros;
// BUNDLE:SA-BEGIN
pub mod annealing;
// BUNDLE:SA-END
// BUNDLE:BEAM-BEGIN
pub mod beam;
// BUNDLE:BEAM-END
// BUNDLE:GREEDY-BEGIN
pub mod greedy;
// BUNDLE:GREEDY-END
// BUNDLE:HILLCLIMB-BEGIN
pub mod hill_climb;
// BUNDLE:HILLCLIMB-END
// BUNDLE:NAIVE-BEGIN
pub mod naive;
// BUNDLE:NAIVE-END
pub mod options;
pub mod traits;
pub mod util;

use crate::problem::{self, Input, State};

#[derive(Clone, Copy, Debug)]
pub enum SolverType {
    SA,
    HillClimb,
    Greedy,
    Beam,
    Naive,
}

pub fn solve(input: &Input, config: &options::Config) -> State {
    // 問題依存の初期解は problem::State::new 側で構築する。
    let initial = State::new(input);
    let budget =
        util::TimeBudget::new(config.search.time_limit_sec, config.search.reserve_time_sec);
    let main_timer = budget.main_timer();

    let best = match config.solver.solver_type {
        // BUNDLE:SA-BEGIN
        SolverType::SA => {
            // 焼きなまし: 時間いっぱいまで近傍遷移を試す。
            let mut rng = util::XorShift::new(config.search.seed);
            annealing::run(
                initial,
                input,
                &mut rng,
                &main_timer,
                config.search.start_temp,
                config.search.end_temp,
                config.search.maximize,
            )
        }
        // BUNDLE:SA-END
        // BUNDLE:HILLCLIMB-BEGIN
        SolverType::HillClimb => {
            // 山登り: 改善遷移のみ受理する。
            let mut rng = util::XorShift::new(config.search.seed);
            hill_climb::run(
                initial,
                input,
                &mut rng,
                &main_timer,
                config.search.maximize,
            )
        }
        // BUNDLE:HILLCLIMB-END
        // BUNDLE:GREEDY-BEGIN
        SolverType::Greedy => {
            // 貪欲: 各ステップで最良の1手を選ぶ。
            greedy::run(
                initial,
                input,
                &main_timer,
                config.search.greedy_depth,
                config.search.maximize,
            )
        }
        // BUNDLE:GREEDY-END
        // BUNDLE:BEAM-BEGIN
        // ビームサーチ: 深さと幅を固定して段階的に展開する。
        SolverType::Beam => beam::run(
            initial,
            input,
            &main_timer,
            config.solver.beam_width,
            config.solver.beam_depth,
            config.search.maximize,
        ),
        // BUNDLE:BEAM-END
        // BUNDLE:NAIVE-BEGIN
        // 愚直解・直接構築: problem.rs の NaiveState::solve_naive に書いた解法をそのまま使う。
        SolverType::Naive => naive::run(initial, input, &main_timer),
        // BUNDLE:NAIVE-END
        // BUNDLE:FALLBACK
    };

    let polish_timer = budget.reserve_timer();
    problem::postprocess(best, input, &polish_timer)
}
