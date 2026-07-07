use anyhow::Result;
use clap::Args;

use crate::config::Config;
use crate::git;

#[derive(Args)]
pub struct BrowseArgs {
    #[arg(long, help = "Print the resolved URL instead of opening a browser")]
    pub print: bool,
}

pub fn run(args: BrowseArgs) -> Result<()> {
    let config = Config::load()?;
    let url = git::remote_web_url(&config.data_dir(), &config.git.remote)?;

    if args.print {
        println!("{url}");
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(&url).spawn()?;
    }

    println!("Opening {url}");

    Ok(())
}
