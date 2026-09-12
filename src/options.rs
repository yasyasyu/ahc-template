use crate::{
    SolverType, DEFAULT_BEAM_DEPTH, DEFAULT_BEAM_WIDTH, DEFAULT_END_TEMP, DEFAULT_GREEDY_DEPTH,
    DEFAULT_RESERVE_TIME_SEC, DEFAULT_SEED, DEFAULT_START_TEMP, DEFAULT_TIME_LIMIT_SEC,
    SOLVER_TYPE,
};

#[derive(Clone, Copy, Debug)]
pub struct SearchConfig {
    pub time_limit_sec: f64,
    pub reserve_time_sec: f64,
    pub start_temp: f64,
    pub end_temp: f64,
    pub greedy_depth: usize,
    pub seed: u64,
    pub maximize: bool,
}

impl SearchConfig {
    fn defaults() -> Self {
        Self {
            time_limit_sec: DEFAULT_TIME_LIMIT_SEC,
            reserve_time_sec: DEFAULT_RESERVE_TIME_SEC,
            start_temp: DEFAULT_START_TEMP,
            end_temp: DEFAULT_END_TEMP,
            greedy_depth: DEFAULT_GREEDY_DEPTH,
            seed: DEFAULT_SEED,
            maximize: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SolverConfig {
    pub solver_type: SolverType,
    pub beam_width: usize,
    pub beam_depth: usize,
}

impl SolverConfig {
    fn defaults() -> Self {
        Self {
            solver_type: SOLVER_TYPE,
            beam_width: DEFAULT_BEAM_WIDTH,
            beam_depth: DEFAULT_BEAM_DEPTH,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub search: SearchConfig,
    pub solver: SolverConfig,
}

impl Config {
    fn defaults() -> Self {
        Self {
            search: SearchConfig::defaults(),
            solver: SolverConfig::defaults(),
        }
    }

    pub fn load() -> Self {
        #[cfg(feature = "local-params")]
        if let Ok(content) = std::fs::read_to_string("params.toml") {
            return Self::parse(&content);
        }

        Self::defaults()
    }

    fn parse(content: &str) -> Self {
        let mut config = Self::defaults();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim().split('#').next().unwrap_or("").trim();

            match key {
                "time_limit_sec" => {
                    if let Ok(parsed) = value.parse() {
                        config.search.time_limit_sec = parsed;
                    }
                }
                "reserve_time_sec" => {
                    if let Ok(parsed) = value.parse() {
                        config.search.reserve_time_sec = parsed;
                    }
                }
                "start_temp" => {
                    if let Ok(parsed) = value.parse() {
                        config.search.start_temp = parsed;
                    }
                }
                "end_temp" => {
                    if let Ok(parsed) = value.parse() {
                        config.search.end_temp = parsed;
                    }
                }
                "greedy_depth" => {
                    if let Ok(parsed) = value.parse() {
                        config.search.greedy_depth = parsed;
                    }
                }
                "seed" => {
                    if let Ok(parsed) = value.parse() {
                        config.search.seed = parsed;
                    }
                }
                "maximize" => {
                    if let Ok(parsed) = value.parse() {
                        config.search.maximize = parsed;
                    }
                }
                "beam_width" => {
                    if let Ok(parsed) = value.parse() {
                        config.solver.beam_width = parsed;
                    }
                }
                "beam_depth" => {
                    if let Ok(parsed) = value.parse() {
                        config.solver.beam_depth = parsed;
                    }
                }
                "solver_type" => {
                    config.solver.solver_type = match value {
                        "sa" | "SA" => SolverType::SA,
                        "hill_climb" | "hillclimb" | "HC" => SolverType::HillClimb,
                        "greedy" | "GREEDY" => SolverType::Greedy,
                        "beam" | "BEAM" => SolverType::Beam,
                        _ => config.solver.solver_type,
                    };
                }
                _ => {}
            }
        }

        config
    }
}
