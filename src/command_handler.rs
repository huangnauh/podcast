use clap::{App, ArgMatches};

use crate::actions::*;
use crate::arg_parser;
use crate::commands;
use crate::structs::*;
use anyhow::Result;

pub fn parse_sub_command(matches: &ArgMatches) -> commands::Command {
    match matches.subcommand_name() {
        Some("download") => commands::Command::Download,
        Some("ls") | Some("list") => commands::Command::List,
        Some("play") => commands::Command::Play,
        Some("sub") | Some("subscribe") => commands::Command::Subscribe,
        Some("search") => commands::Command::Search,
        Some("rm") => commands::Command::Remove,
        Some("completion") => commands::Command::Complete,
        Some("refresh") => commands::Command::Refresh,
        Some("update") => commands::Command::Update,
        _ => commands::Command::NoMatch,
    }
}

pub async fn handle_matches(
    version: &str,
    client: &reqwest::Client,
    state: &mut State,
    config: Config,
    app: &mut App<'_, '_>,
    matches: &ArgMatches<'_>,
) -> Result<()> {
    let command = parse_sub_command(matches);
    match command {
        commands::Command::Download => {
            arg_parser::download(&client, state, matches).await?;
        }
        commands::Command::List => {
            arg_parser::list(state, matches)?;
        }
        commands::Command::Play => {
            arg_parser::play(state, matches)?;
        }
        commands::Command::Subscribe => {
            arg_parser::subscribe(&client, state, config, matches).await?;
        }
        commands::Command::Search => {
            arg_parser::search(&client, state, config, matches).await?;
        }
        commands::Command::Remove => {
            arg_parser::remove(state, matches)?;
        }
        commands::Command::Complete => {
            arg_parser::complete(app, matches)?;
        }
        commands::Command::Refresh => {
            update_rss(&client, state, Some(config)).await?;
        }
        commands::Command::Update => {
            check_for_update(version).await?;
        }
        _ => (),
    };
    Ok(())
}
