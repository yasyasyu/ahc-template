#![allow(dead_code, non_snake_case, unused_macros)]

// 各アルゴリズムモジュールは src/ 以下に分割してある。
// 提出時に不要なものだけを削って1ファイルにまとめたい場合は tools/bundle.ps1 を使う。
// (mod 宣言・solve() 内の対応する分岐は `// BUNDLE:<KEY>-BEGIN`〜`-END` で囲ってある)
// BUNDLE:SA-BEGIN
mod annealing;
// BUNDLE:SA-END
// BUNDLE:BEAM-BEGIN
mod beam;
// BUNDLE:BEAM-END
// BUNDLE:GREEDY-BEGIN
mod greedy;
// BUNDLE:GREEDY-END
// BUNDLE:HILLCLIMB-BEGIN
mod hill_climb;
// BUNDLE:HILLCLIMB-END
mod options;
mod problem;
mod util;

const DEFAULT_TIME_LIMIT_SEC: f64 = 1.95;
const DEFAULT_RESERVE_TIME_SEC: f64 = 0.03;

const DEFAULT_START_TEMP: f64 = 1e4;
const DEFAULT_END_TEMP: f64 = 1e1;
const DEFAULT_BEAM_WIDTH: usize = 200;
const DEFAULT_BEAM_DEPTH: usize = 60;
const DEFAULT_GREEDY_DEPTH: usize = 1_000_000;
const DEFAULT_SEED: u64 = 0; // 乱数シード。0 のときはデフォルト値を使う。

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        if cfg!(feature = "local") {
            eprintln!($($arg)*);
        }
    };
}

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

const SOLVER_TYPE: SolverType = SolverType::SA;

#[derive(Clone, Copy, Debug)]
enum SolverType {
    SA,
    HillClimb,
    Greedy,
    Beam,
}

fn main() {
    let config = options::Config::load();
    let input = problem::Input::read();
    let best: problem::State = solve(&input, &config);
    problem::output(&best, &input);
}

fn solve(input: &problem::Input, config: &options::Config) -> problem::State {
    // 問題依存の初期解は problem::State::new 側で構築する。
    let initial = problem::State::new(input);
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
            hill_climb::run(initial, input, &mut rng, &main_timer, config.search.maximize)
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
        // BUNDLE:FALLBACK
    };

    let polish_timer = budget.reserve_timer();
    problem::postprocess(best, input, &polish_timer)
}
