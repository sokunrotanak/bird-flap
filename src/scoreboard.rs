use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use anyhow::{bail, Context, Result, Error};
use dotenvy::dotenv;
use std::{env, time::Duration};
use ehttp::{*, Response};
use dotenvy_macro::dotenv;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::world::menu::TEXT_COLOR;

#[derive(Resource, Default)]
pub struct ScoreboardLog {
    pub rows: Vec<ScoreRow>,
    pub status: LoadStatus,
    pub last_submitted_id: Option<i64>,
}

#[derive(Default)]
pub enum LoadStatus {
    #[default]
    NotLoaded,
    Loading,
    Loaded,
    Failed(String)
}

#[derive(Resource)]
pub struct ScoreboardChannel {
    pub tx: Sender<ScoreboardMsg>,
    pub rx: Receiver<ScoreboardMsg>,
}

impl Default for ScoreboardChannel {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        ScoreboardChannel{tx, rx}

    }
}

pub enum ScoreboardMsg {
    Fetched(Result<Vec<ScoreRow>>),
    Submitted(Result<ScoreRow>)
}

#[derive(Event)]
pub struct ScoreSubmitted{pub id: i64}

pub struct ScoreboardPlugin;

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScoreboardChannel>();
        app.add_systems(Update, (
            // trigger_fetch,
            poll_results,
//            trigger_submit,
        ));
        app.init_resource::<ScoreboardLog>();
        app.init_resource::<SubmitStatus>();

    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ScoreRow {
    pub id: i64,
    pub nickname: String,
    pub score: u32,
    pub game_mode: String,
    pub submitted_at: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitScore {
    pub nickname: String,
    pub score: u32,
    pub game_mode: String,
}

// #[derive(Resource, Default)]
// pub struct NicknameForm {
//     pub text: String,
//     pub status: SubmitStatus,
// }

#[derive(Default, Resource, PartialEq)]
pub enum SubmitStatus {
    #[default]
    Editing,
    Submitting,
    Failed(String)
}

struct Config {
    supabase_url: &'static str,
    api_key: &'static str,
}

const CONFIG: Config = Config {
    supabase_url: dotenv!("SUPABASE_URL"),
    api_key: dotenv!("API_KEY"),
};

// Handle error response
fn ensure_success(response: Response) -> Result<Response, anyhow::Error> {
    if response.ok {
        return Ok(response)
    } else {
        let status = response.status;
        let body = response.text().unwrap_or_default().to_string();
        bail!("Request failed with {status}: {body}");
    }
}

async fn fetch_top_scores(
    limit: u32,
)  -> Result<Vec<ScoreRow>> {
    let full_url = format!("{}?select=*&limit={}&order=score.desc", CONFIG.supabase_url, limit);
    let request = Request{
        headers: Headers::new(&[
            ("Accept", "application/json"),
            ("apikey", CONFIG.api_key),
        ]),
        timeout: Some(Duration::from_secs(10)),
        ..Request::get(full_url)
    };

    let response = fetch_async(request)
        .await
        .map_err(anyhow::Error::msg)
        .context("could not send the scoreboard request")?;

    ensure_success(response)
        .context("fetching scoreboard")?
        .json::<Vec<ScoreRow>>()
        .context("could not parse the scoreboard response")

}

async fn submit_score(
    entry: &SubmitScore,
) -> Result<ScoreRow> {
    let full_url = format!("{}", CONFIG.supabase_url);
    let request = Request{
        headers: Headers::new(&[
            ("Accept", "application/vnd.pgrst.object+json"),
            ("Prefer", "return=representation"),
            ("apikey", CONFIG.api_key),
        ]),
        timeout: Some(Duration::from_secs(10)),
        ..Request::post_json(full_url, entry).context("could not serialize the score")?
    };
    let response = fetch_async(request)
        .await
        .map_err(anyhow::Error::msg)
        .context("could not submit the score")?;

    ensure_success(response)
        .context("Submitting score")?
        .json::<ScoreRow>()
        .context("could not parse the submit response")
}

pub fn trigger_fetch (
    tx: Sender<ScoreboardMsg>,
) {
    //let tx = channel.tx.clone();
    IoTaskPool::get()
        .spawn(async move {
            let result = fetch_top_scores( 10).await;
            let _ = tx.send(ScoreboardMsg::Fetched(result));
        })
        .detach();
}

fn poll_results(
    channel: Res<ScoreboardChannel>,
    mut scoreboard: ResMut<ScoreboardLog>,
    mut status: ResMut<SubmitStatus>,
    mut commands: Commands,
) {
    for msg in channel.rx.try_iter() {
        match msg {
            ScoreboardMsg::Fetched(Ok(scores)) => {
                scoreboard.rows = scores;
                scoreboard.status = LoadStatus::Loaded;
            },
            ScoreboardMsg::Fetched(Err(e)) => {
                scoreboard.status = LoadStatus::Failed(format!("{e:#}"));
            },
            ScoreboardMsg::Submitted(Ok(row)) => {commands.trigger(ScoreSubmitted {id: row.id})},
            ScoreboardMsg::Submitted(Err(e)) => { *status = SubmitStatus::Failed(format!("{e:#}")); warn!( "failed {e:#}") },
        }
    }
}

pub fn request_submit(
    tx: Sender<ScoreboardMsg>,
    entry: SubmitScore,
) {
    IoTaskPool::get()
        .spawn(async move {
            let result = submit_score(&entry).await;
            let _ = tx.send(ScoreboardMsg::Submitted(result));
        })
        .detach();
}