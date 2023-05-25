use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use edgedb_protocol::model::{Datetime, Duration, Uuid};
use edgedb_tokio::{Client, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Config {
    pub user_id: Option<String>,
}

fn find_config_location() -> Result<PathBuf> {
    let mut config_dir = dirs::config_dir().ok_or(anyhow!("Cannot find config location."))?;

    config_dir.push("time-tracker-edge.toml");
    Ok(config_dir)
}

fn read_config() -> Result<Config> {
    let config = std::fs::read_to_string(find_config_location()?);

    match config {
        Ok(config) => {
            let config = toml::from_str::<Config>(&config)?;
            Ok(config)
        }
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => Ok(Config::default()),
            _ => Err(err.into()),
        },
    }
}

fn write_config(config: &Config) -> Result<()> {
    let config_toml = toml::to_string(config)?;

    std::fs::write(find_config_location()?, config_toml)?;
    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
struct State {
    pub config: Config,
    pub user_id: Uuid,

    pub client: Client,
}

#[derive(Queryable)]
#[allow(dead_code)]
struct User {
    pub id: Uuid,
    pub password: String,
}

#[derive(Queryable)]
#[allow(dead_code)]
struct Project {
    pub id: Uuid,
    pub name: String,
    pub is_default: bool,
}

#[derive(Queryable)]
#[allow(dead_code)]
struct Entry {
    pub id: Uuid,
    pub start_at: Datetime,
    pub stop_at: Option<Datetime>,
    pub duration: Duration,
    pub project_name: String,
}

#[derive(Parser)]
enum Cli {
    /// Starts time tracker
    Start {
        project: Option<String>,
    },

    /// Stops time tracker
    Stop,

    /// Lists all entries
    List,

    /// Manage projects
    #[command(subcommand)]
    Project(CliProject),

    Login {
        password: String,
    },
    Logout,
}

#[derive(Subcommand)]
enum CliProject {
    List,

    Add { name: String },

    Remove { name: String },

    /// Set default project
    Default { name: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut config = read_config()?;

    match cli {
        Cli::Login { password } => {
            let user_id = login(&password).await?;
            config.user_id = Some(user_id.to_string());
            write_config(&config)?;
            println!("Successfully logged in.");
            return Ok(());
        }
        Cli::Logout => {
            config.user_id = None;
            write_config(&config)?;
            return Ok(());
        }
        _ => (),
    }

    let user_id = config
        .user_id
        .as_ref()
        .and_then(|s| Uuid::from_str(s).ok())
        .ok_or_else(|| anyhow!("Not logged in. Run `tte login`."))?;

    let client = edgedb_tokio::create_client().await?;

    // validate that user still exists
    let found = client
        .query_single::<Uuid, _>(
            "SELECT (SELECT User FILTER .id = <uuid>$0).id",
            &(&user_id,),
        )
        .await?;
    if found.is_none() {
        return Err(anyhow!(
            "Your account has been deleted. Create new account by running `tte login`."
        ));
    }

    let state = State {
        config,
        client,
        user_id,
    };

    match cli {
        Cli::Login { .. } | Cli::Logout => unreachable!(),

        Cli::Start { project } => {
            stop(&state).await?;
            start(project, &state).await?;

            list_entries(&state).await?;
        }

        Cli::Stop => {
            stop(&state).await?;

            list_entries(&state).await?;
        }
        Cli::List => {
            list_entries(&state).await?;
        }

        Cli::Project(cli) => match cli {
            CliProject::List => {
                list_projects(&state).await?;
            }
            CliProject::Add { name } => {
                state
                    .client
                    .query_required_single_json(
                        "INSERT Project { name := <str>$0, owner := (SELECT User FILTER .id = <uuid>$1) }",
                        &(name.as_str(), &state.user_id ),
                    )
                    .await?;
                list_projects(&state).await?;
            }
            CliProject::Remove { name } => {
                let deleted = state
                    .client
                    .query_required_single::<i64, _>(
                        "select count((DELETE Project FILTER .name = <str>$0 and .owner.id = <uuid>$1))",
                        &(name.as_str(), &state.user_id),
                    )
                    .await?;
                println!("Deleted {deleted} projects.\n");
                list_projects(&state).await?;
            }
            CliProject::Default { name } => {
                state
                    .client
                    .query_required_single_json(
                        "UPDATE User
                        FILTER .id = <uuid>$1
                        SET { 
                            default_project := (
                                SELECT Project 
                                FILTER .name = <str>$0 AND .owner.id = <uuid>$1
                                LIMIT 1
                            )
                        }",
                        &(name.as_str(), &state.user_id),
                    )
                    .await?;

                list_projects(&state).await?;
            }
        },
    }
    Ok(())
}

async fn login(password: &str) -> Result<Uuid> {
    let edb = edgedb_tokio::create_client().await?;

    // find existing user
    let user = edb
        .query_single::<User, _>(
            "SELECT User { id, password } FILTER .password = <str>$0 LIMIT 1",
            &(password,),
        )
        .await?;

    if let Some(user) = user {
        return Ok(user.id);
    }

    // create user
    let user = edb
        .query_required_single::<User, _>(
            "SELECT (INSERT User { password := <str>$0 }) { id, password }",
            &(password,),
        )
        .await?;

    Ok(user.id)
}

async fn list_projects(state: &State) -> Result<()> {
    let projects = state
        .client
        .query::<Project, _>(
            "SELECT Project { id, name, is_default := exists(.<default_project[is User]) }
            FILTER .owner.id = <uuid>$0",
            &(state.user_id,),
        )
        .await?;

    println!("Projects:");
    for project in projects {
        let annotation = if project.is_default { " (*)" } else { "" };
        println!("{}{}", project.name, annotation);
    }
    Ok(())
}

async fn start(project: Option<String>, state: &State) -> Result<()> {
    state
        .client
        .query_required_single_json(
            "INSERT Entry {
            start_at := datetime_of_statement(),
            project := assert_single(
                assert_exists((
                    SELECT Project
                    FILTER .owner.id = <uuid>$1
                        AND (
                        # was specified by name
                        .name ?= <optional str>$0
                        
                        # is default
                        OR (
                            not exists(<optional str>$0)
                            AND exists(.<default_project[is User])
                        )
                    )
                ), message := 'project not found'
            ), message := 'multiple projects with that name')
        }",
            &(project, &state.user_id),
        )
        .await?;

    Ok(())
}
async fn stop(state: &State) -> Result<()> {
    state
        .client
        .query_json(
            "UPDATE Entry
                    FILTER .project.owner.id = <uuid>$0 AND NOT exists(.stop_at)
                    SET {
                        stop_at := datetime_of_statement(),
                    }",
            &(&state.user_id,),
        )
        .await?;

    Ok(())
}

async fn list_entries(state: &State) -> Result<()> {
    let entries = state
        .client
        .query::<Entry, _>(
            "SELECT Entry { id, start_at, stop_at,
                duration := (.stop_at ?? datetime_of_statement()) - .start_at,
                project_name := .project.name
            }
            FILTER .project.owner.id = <uuid>$0",
            &(state.user_id,),
        )
        .await?;

    println!("Entries:");
    println!(
        "{:30} | {:30} | {:12} | {:20}",
        "start", "stop", "duration", "project"
    );
    println!("{:-<30}-|-{:-<30}-|-{:-<12}-|-{:-<20}", "", "", "", "");
    for entry in entries {
        let stop_at = entry.stop_at.map(|s| s.to_string()).unwrap_or_default();

        // round to sec
        let mut duration = entry.duration.to_micros();
        duration -= duration % 1000000;
        let duration = Duration::from_micros(duration).to_string();

        println!(
            "{:30} | {:30} | {:>12} | {:20}",
            entry.start_at, stop_at, duration, entry.project_name
        );
    }
    Ok(())
}
