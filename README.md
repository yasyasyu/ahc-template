## usage
`cargo generate --git https://github.com/yasyasyu/ahc-template.git --name ahc`

## minimum tasks
まず埋める必要があるものを、共通部分とソルバ別に分けて整理します。

共通で必要:
- `problem::Input` に問題の不変データを持たせて `read` を実装する
- `problem::State` に状態表現を、`problem::Neighbor` に遷移表現を入れる
- `State::new` で初期解を構築する
- `State::score` で評価関数を実装する
- `problem::output` で提出形式の出力を実装する

`SA` / `HillClimb` を使うなら追加で必要:
- `State::neighbor`
- `State::apply_neighbor`
- `State::rollback_neighbor`

`Greedy` / `Beam` を使うなら追加で必要:
- `State::enumerate_actions`
- `State::apply_action`
- `State::is_terminal`

`Beam` を使うならさらに必要:
- `State::dedup_key`

任意の拡張ポイント:
- `State::enumerate_actions_with_timer`: 列挙途中で時間を見て打ち切りたいときに使う
- `problem::postprocess`: 本探索のあとに余り時間で解を磨きたいときに使う
- `options::Config` と `params.toml`: 定数を外出ししてローカル調整したいときだけ使う

最小構成で始めるなら、使うソルバを 1 つだけ決めて、その系統の必須メソッドだけ先に埋めるのがおすすめです。

## local tuning
通常実行:
`cargo run --release`

ローカル設定ファイルを有効にする場合:
`cargo run --release --features local-params`

`params.toml` をルートに置くと、次のキーでテンプレートの探索設定を上書きできます。

```toml
time_limit_sec = 1.95
reserve_time_sec = 0.03
start_temp = 10000.0
end_temp = 10.0
greedy_depth = 1000000
seed = 0
maximize = true
beam_width = 200
beam_depth = 60
solver_type = "sa"
```

`solver_type` は `sa`, `hill_climb`, `greedy`, `beam` を受け付けます。
テンプレート側では `reserve_time_sec` を終盤の postprocess 用に確保し、`problem::postprocess` で余り時間を使えるようにしています。

## utility helpers
テンプレートには、問題依存コードを書き始める前にそのまま使える補助をいくつか入れています。

- `util::TimeBudget`: 本探索と postprocess の時間を分けるための薄いラッパ
- `util::TimeKeeper::multi_start_deadlines(n)`: 残り時間を `n` 回の再試行に均等配分する deadline 列
- `util::Track<T>`: 親ポインタ付きの復元用トラック
- `static_data!`: `OnceLock` ベースの前計算キャッシュ
- `util::XorShift::random_usize` / `shuffle`: 近傍生成や候補順シャッフル用の軽量補助

例:

```rust
let budget = util::TimeBudget::new(1.95, 0.03);
let main_timer = budget.main_timer();

for deadline in main_timer.multi_start_deadlines(3) {
	if main_timer.is_over_elapsed(deadline) {
		break;
	}
}

let mut track = util::Track::new();
let root = track.push(!0, 0usize);
let leaf = track.push(root, 1usize);
let path = track.restore(leaf);

static_data! {
	get_table:Vec<usize>| (0..10).collect()
}
```
