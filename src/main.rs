#![allow(dead_code, non_snake_case, unused_macros)]

// テンプレート: https://github.com/yasyasyu/ahc-template
//
// 問題ごとに触るのは次の2ファイルだけ:
// - src/main.rs    : ソルバ選択と探索パラメータの既定値 (このファイル)
// - src/problem.rs : 入力・状態・評価関数・遷移・出力
// src/engine/ 以下は探索アルゴリズムなどの共通基盤なので触らなくてよい。
//
// 提出時に不要なアルゴリズムを削って1ファイルにまとめたい場合は tools/bundle.ps1 を使う。

#[macro_use]
mod engine;
mod problem;

use crate::engine::SolverType;

const SOLVER_TYPE: SolverType = SolverType::SA;

const DEFAULT_TIME_LIMIT_SEC: f64 = 1.95;
const DEFAULT_RESERVE_TIME_SEC: f64 = 0.03;
const DEFAULT_MAXIMIZE: bool = true; // 評価値が大きいほど良いなら true

const DEFAULT_START_TEMP: f64 = 1e4;
const DEFAULT_END_TEMP: f64 = 1e1;
const DEFAULT_BEAM_WIDTH: usize = 200;
const DEFAULT_BEAM_DEPTH: usize = 60;
const DEFAULT_GREEDY_DEPTH: usize = 1_000_000;
const DEFAULT_SEED: u64 = 0; // 乱数シード。0 のときはデフォルト値を使う。

fn main() {
    let config = engine::options::Config::load();
    let input = problem::Input::read();
    let best = engine::solve(&input, &config);
    problem::output(&best, &input);
}
