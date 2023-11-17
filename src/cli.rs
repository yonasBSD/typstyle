use std::path::PathBuf;

use clap::Parser;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Parser)]
#[clap(name = "typst-geshihua", author, version, about, long_version(LONG_VERSION.as_str()))]
pub struct CliArguments {
    pub input: PathBuf,
}

static NONE: &str = "None";
static LONG_VERSION: Lazy<String> = Lazy::new(|| {
    format!(
        "
Build Timestamp:     {}
Build Git Describe:  {}
Commit SHA:          {}
Commit Date:         {}
Commit Branch:       {}
Cargo Target Triple: {}
",
        env!("VERGEN_BUILD_TIMESTAMP"),
        env!("VERGEN_GIT_DESCRIBE"),
        option_env!("VERGEN_GIT_SHA").unwrap_or(NONE),
        option_env!("VERGEN_GIT_COMMIT_TIMESTAMP").unwrap_or(NONE),
        option_env!("VERGEN_GIT_BRANCH").unwrap_or(NONE),
        env!("VERGEN_CARGO_TARGET_TRIPLE"),
    )
});