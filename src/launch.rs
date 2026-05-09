use std::env;

use crate::preview::PreviewRequest;

const DEFAULT_PREVIEW_SEED: u64 = 0xA5C1_1B10;

pub struct LaunchOptions {
    pub balance_report: bool,
    pub preview_request: PreviewRequest,
    pub seed: u64,
}

impl LaunchOptions {
    pub fn from_env() -> Self {
        Self::from_args_with_seed_source(env::args().skip(1), ::rand::random)
    }

    fn from_args_with_seed_source<I, F>(args: I, fresh_seed: F) -> Self
    where
        I: Iterator<Item = String>,
        F: FnOnce() -> u64,
    {
        let args = args.collect::<Vec<_>>();
        let balance_report = args.iter().any(|arg| arg == "--balance-report");
        let preview_request = PreviewRequest::from_args(args.clone().into_iter());
        let seed = explicit_seed(&args).unwrap_or_else(|| {
            if preview_request.is_preview() || balance_report {
                DEFAULT_PREVIEW_SEED
            } else {
                fresh_seed()
            }
        });
        Self {
            balance_report,
            preview_request,
            seed,
        }
    }
}

fn explicit_seed(args: &[String]) -> Option<u64> {
    args.windows(2)
        .find(|pair| pair[0] == "--seed")
        .and_then(|pair| parse_seed(&pair[1]))
}

fn parse_seed(value: &str) -> Option<u64> {
    let value = value.replace('_', "");
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).ok()
    } else {
        value.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preview::PreviewMode;
    use std::path::PathBuf;

    #[test]
    fn normal_runs_use_a_fresh_seed_when_none_is_provided() {
        let options =
            LaunchOptions::from_args_with_seed_source(std::iter::empty::<String>(), || 777);
        assert_eq!(options.seed, 777);
        assert!(!options.balance_report);
        assert!(matches!(options.preview_request, PreviewRequest::None));
    }

    #[test]
    fn previews_keep_the_stable_default_seed() {
        let options = LaunchOptions::from_args_with_seed_source(
            ["--preview", "inventory", "--output", "/tmp/inventory.png"]
                .into_iter()
                .map(String::from),
            || 777,
        );
        assert_eq!(options.seed, DEFAULT_PREVIEW_SEED);
        assert!(!options.balance_report);
        assert!(matches!(
            options.preview_request,
            PreviewRequest::Single {
                mode: PreviewMode::Inventory,
                path
            } if path == PathBuf::from("/tmp/inventory.png")
        ));
    }

    #[test]
    fn explicit_seed_overrides_normal_and_preview_defaults() {
        let decimal = LaunchOptions::from_args_with_seed_source(
            ["--seed", "42"].into_iter().map(String::from),
            || 777,
        );
        assert_eq!(decimal.seed, 42);

        let hex_preview = LaunchOptions::from_args_with_seed_source(
            [
                "--preview",
                "inventory",
                "--output",
                "/tmp/inventory.png",
                "--seed",
                "0xA5C1_1B10",
            ]
            .into_iter()
            .map(String::from),
            || 777,
        );
        assert_eq!(hex_preview.seed, DEFAULT_PREVIEW_SEED);
    }

    #[test]
    fn balance_reports_use_the_stable_default_seed() {
        let options = LaunchOptions::from_args_with_seed_source(
            ["--balance-report"].into_iter().map(String::from),
            || 777,
        );
        assert_eq!(options.seed, DEFAULT_PREVIEW_SEED);
        assert!(options.balance_report);
    }
}
