use super::types::{self, GameInfo, Role};
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use std::convert::TryFrom;
use uuid::Uuid;

struct Player {
    name: String,
    role: String,
}

pub async fn create_game(pool: &SqlitePool, game: &GameInfo) -> sqlx::Result<()> {
    let mut txn = pool.begin().await?;
    let game_id = Uuid::new_v4().to_string();
    sqlx::query!(
        "insert into games (id, winner) VALUES (?, ?)",
        game_id,
        game.result.winner
    )
    .execute(&mut txn)
    .await?;
    for (player, role) in &game.players {
        sqlx::query!(
            "INSERT INTO player_roles VALUES (?, ?, ?)",
            game_id,
            player,
            role
        )
        .execute(&mut txn)
        .await?;
    }

    for quest in &game.quests {
        let quest_id = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO quests (id, fails, status) VALUES (?, ?, ?)",
            quest_id,
            quest.fails.unwrap_or(0),
            &quest.status,
        )
        .execute(&mut txn)
        .await?;
        sqlx::query!(
            "INSERT INTO games_to_quests VALUES (?, ?)",
            game_id,
            quest_id,
        )
        .execute(&mut txn)
        .await?;
        for participant in &quest.participants {
            sqlx::query!(
                "INSERT INTO quest_participants (quest_id, name, role) VALUES (?, ?, ?)",
                quest_id,
                participant,
                game.players.get(participant).unwrap_or(&Role::Servant)
            )
            .execute(&mut txn)
            .await?;
        }
    }
    txn.commit().await?;
    Ok(())
}

pub async fn find_game(pool: &SqlitePool, game_id: &str) -> sqlx::Result<GameInfo> {
    let raw_winner = sqlx::query!("SELECT winner FROM games WHERE id = ?", game_id)
        .fetch_one(pool)
        .await?
        .winner;
    let mut player_records = sqlx::query_as!(
        Player,
        "SELECT name, role FROM player_roles WHERE game_id = ?",
        game_id
    )
    .fetch_all(pool)
    .await?;
    let mut players = HashMap::<String, Role>::new();
    for row in player_records.drain(..) {
        players.insert(row.name, Role::try_from(row.role.as_str())?);
    }

    let quest_ids = sqlx::query!(
        "SELECT quest_id from games_to_quests WHERE game_id = ?",
        game_id
    )
    .fetch_all(pool)
    .await?;

    let mut quests = Vec::new();
    for ids in quest_ids {
        let id = ids.quest_id.unwrap_or_else(String::new);
        let quest = sqlx::query!("SELECT fails, status FROM quests where id = ?", id)
            .fetch_one(pool)
            .await?;
        let all_participants =
            sqlx::query!("SELECT name from quest_participants where quest_id = ?", id)
                .fetch_all(pool)
                .await?;

        let mut participant_names = Vec::new();
        for p in all_participants {
            participant_names.push(p.name.unwrap());
        }

        quests.push(types::Quest {
            status: types::QuestStatus::Success,
            fails: quest.fails,
            participants: participant_names,
        });
    }

    let num_failures = quests
        .iter()
        .filter(|q| q.status == types::QuestStatus::Fail)
        .count();

    let winner = if raw_winner == Some("good".into()) {
        types::Alignment::Good
    } else {
        types::Alignment::Evil
    };

    let victory_type = if num_failures < 3 && winner == types::Alignment::Evil {
        types::VictoryType::Assassination
    } else {
        types::VictoryType::Quest
    };

    Ok(GameInfo {
        players,
        quests,
        result: types::EndResult {
            winner,
            victory_type,
        },
    })
}

pub async fn load_all_games(pool: &SqlitePool) -> sqlx::Result<Vec<GameInfo>> {
    let mut games = Vec::new();
    let records = sqlx::query!("SELECT id FROM games").fetch_all(pool).await?;
    for record in records {
        games.push(find_game(pool, record.id.unwrap().as_str()).await?);
    }
    Ok(games)
}
