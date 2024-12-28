use anyhow::Result;
use tui::{control_panel::App, setup_panel::setup_tui, tui::TUI};

// 1. Bring in clap
use clap::Parser;


mod tui;
mod widgets;

#[derive(Parser, Debug)]
#[command(author, version, about = "Setup TUI Example", long_about = None)]
struct CliArgs {
    #[arg(long)]
    url: Option<String>,

    #[arg(long)]
    token: Option<String>,
}


fn main() -> Result<()> {
    // 1) Parse command-line
    let args = CliArgs::parse();
    let mut tui = TUI::new();

    let (url, token) = match (args.url, args.token) {
        (Some(url), Some(token)) => (url, token),
        (url, token) => setup_tui(url, token)?
    };

    let app = App::from((url, token));
    tui.run(app)?;
    Ok(())
}



