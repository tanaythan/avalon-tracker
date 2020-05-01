mod db;
pub mod types;

use std::env;
use std::path::PathBuf;

use db::{create_game, find_game, load_all_games};
use sqlx::sqlite::SqlitePool;
use structopt::StructOpt;

type AllResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "avalon-tracker",
    about = "Tracks avalon games through a sqlite DB"
)]
enum Command {
    Import {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
    Load {
        id: String,
    },
    Standings,
}

#[async_std::main]
async fn main() -> AllResult<()> {
    dotenv::dotenv().ok();
    let pool = SqlitePool::builder()
        .max_size(1)
        .build(
            env::var("DATABASE_URL")
                .expect("DATABASE_URL should be populated")
                .as_str(),
        )
        .await?;
    let opt = Command::from_args();
    match opt {
        Command::Import { file } => {
            let f = std::fs::read_to_string(file)?;
            let games: Vec<types::GameInfo> = serde_yaml::from_str(f.as_str())?;
            for game in &games {
                create_game(&pool, game).await?;
            }
        }
        Command::Load { id } => {
            dbg!(find_game(&pool, id.as_str()).await?);
        }
        Command::Standings => {
            println!("{}", types::standings(&load_all_games(&pool).await?));
        }
    };
    Ok(())
}
