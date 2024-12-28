use anyhow::{Result, anyhow};
use tui::{control_panel::App, setup_panel::SetupPanel, tui::TUI};

// 1. Bring in clap
use clap::Parser;


mod tui;
mod widgets;

#[derive(Parser, Debug)]
#[command(author, version, about = "HiveCore monutor TUI", long_about = None)]
struct CliArgs {
    #[arg(long)]
    url: Option<String>,

    #[arg(long)]
    token: Option<String>,
}


fn main() -> Result<()> {
    let args = CliArgs::parse();
    let mut tui = TUI::new();



    let (url, token) = if args.url.is_none() || args.token.is_none() {
        let mut setup_panel = SetupPanel::new(args.url, args.token);
        tui.run(&mut setup_panel)?;
        let (url, token) = setup_panel.final_values();
        
        if url.is_none() || token.is_none() {
            return Err(anyhow!("Invalid or missing credentials..."));
        }

        (url.unwrap(), token.unwrap())
    } else {
        (args.url.unwrap(), args.token.unwrap())
    };
   
    let mut app = App::from((url, token));
    tui.run(&mut app)
}



