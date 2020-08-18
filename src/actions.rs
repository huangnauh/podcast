use crate::download;
use crate::structs::*;
use crate::utils;
use anyhow::Result;

use futures::prelude::*;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};

use clap::App;
use clap::Shell;
use download::download_episodes;
use regex::Regex;
use reqwest;
use rss::Channel;
use std::path::PathBuf;

pub fn list_episodes(state: &State, search: &str) -> Result<()> {
    let re = Regex::new(&format!("(?i){}", &search))?;
    for subscription in &state.subscriptions {
        if re.is_match(subscription.title()) {
            let mut path = utils::get_xml_dir()?;
            let mut filename: String = subscription.title.clone();
            filename.push_str(".xml");
            path.push(filename);
            let file = File::open(&path)?;
            let channel = Channel::read_from(BufReader::new(file))?;
            let podcast = Podcast::from(channel);
            let mut episodes = podcast.episodes();
            if subscription.reverse {
                episodes.reverse()
            }
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            episodes
                .iter()
                .filter(|ep| ep.title().is_some())
                .enumerate()
                .for_each(|(num, ep)| {
                    writeln!(
                        &mut handle,
                        "({}) {}",
                        episodes.len() - num,
                        ep.title().unwrap()
                    )
                    .ok();
                });
            return Ok(());
        }
    }
    Ok(())
}

pub async fn update_subscription(
    state: &State,
    index: usize,
    sub: &Subscription,
    config: &Config,
) -> Result<[usize; 2]> {
    println!("Updating {}", sub.title);
    let mut path: PathBuf = utils::get_podcast_dir()?;
    path.push(&sub.title);
    utils::create_dir_if_not_exist(&path)?;

    let mut titles = HashSet::new();
    for entry in fs::read_dir(&path)? {
        let unwrapped_entry = &entry?;
        titles.insert(utils::trim_extension(
            &unwrapped_entry.file_name().into_string().unwrap(),
        ));
    }

    let resp = reqwest::get(&sub.url).await?.bytes().await?;
    let podcast = Podcast::from(Channel::read_from(BufReader::new(&resp[..]))?);

    let mut podcast_rss_path = utils::get_xml_dir()?;
    let title = utils::append_extension(podcast.title(), "xml");
    podcast_rss_path.push(title);

    let file = File::create(&podcast_rss_path)?;
    (*podcast).write_to(BufWriter::new(file))?;
    let mut episodes = podcast.episodes();
    if sub.reverse {
        episodes.reverse();
    }
    if sub.num_episodes < episodes.len() {
        let episodes = episodes[..episodes.len() - sub.num_episodes].to_vec();
        let to_download = match config.download_subscription_limit {
            Some(subscription_limit) => {
                let download_futures = episodes
                    .iter()
                    .rev()
                    .take(subscription_limit as usize)
                    .map(|ep| Download::new(&state, &podcast, &ep));

                stream::iter(download_futures)
                    .filter_map(|download| async move { download.await.ok() })
                    .filter_map(|d| async move { d })
                    .collect::<Vec<Download>>()
                    .await
            }
            None => {
                let download_futures = episodes
                    .iter()
                    .map(|ep| Download::new(&state, &podcast, &ep));

                stream::iter(download_futures)
                    .filter_map(|download| async move { download.await.ok() })
                    .filter_map(|d| async move { d })
                    .collect::<Vec<Download>>()
                    .await
            }
        };
        download_episodes(to_download).await?;
    }
    Ok([index, episodes.len()])
}

pub fn list_subscriptions(state: &State) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for subscription in &state.subscriptions {
        writeln!(&mut handle, "{}", subscription.title())?;
    }
    Ok(())
}

pub fn print_completion(app: &mut App, arg: &str) {
    let command_name = "podcast";
    match arg {
        "zsh" => {
            app.gen_completions_to(command_name, Shell::Zsh, &mut io::stdout());
        }
        "bash" => {
            app.gen_completions_to(command_name, Shell::Bash, &mut io::stdout());
        }
        "powershell" => {
            app.gen_completions_to(command_name, Shell::PowerShell, &mut io::stdout());
        }
        "fish" => {
            app.gen_completions_to(command_name, Shell::Fish, &mut io::stdout());
        }
        "elvish" => {
            app.gen_completions_to(command_name, Shell::Elvish, &mut io::stdout());
        }
        other => {
            println!("Completions are not available for {}", other);
        }
    }
}
