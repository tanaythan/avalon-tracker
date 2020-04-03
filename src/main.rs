use serde::Deserialize;
use std::collections::HashMap;

type Players = HashMap<String, Role>;

#[derive(Debug, Deserialize)]
struct GameInfo {
    players: Players,
    quests: Vec<Quest>,
    result: EndResult,
}

#[derive(Debug, Deserialize)]
struct Quest {
    status: QuestStatus,
    fails: i32,
    participants: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EndResult {
    winner: Alignment,
    #[serde(rename = "type")]
    victory_type: VictoryType,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Role {
    Assassin,
    Merlin,
    Minion,
    Mordred,
    Morgana,
    Oberon,
    Percival,
    ReverseOberon,
    Servant,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum QuestStatus {
    Success,
    Fail,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Alignment {
    Good,
    Evil,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum VictoryType {
    Assassination,
    Quest,
}

impl GameInfo {
    pub fn winners(&self) -> Vec<&String> {
        let alignment = self.result.winner;
        self.players
            .iter()
            .filter(|(_, v)| v.alignment() == alignment)
            .map(|(k, _)| k)
            .collect()
    }

    pub fn all_players(&self) -> Vec<&String> {
        self.players.keys().collect()
    }
}

impl Role {
    pub fn alignment(self) -> Alignment {
        use Alignment::*;
        use Role::*;
        match self {
            Assassin | Morgana | Minion | Mordred | Oberon => Evil,
            Merlin | Percival | ReverseOberon | Servant => Good,
        }
    }
}

#[derive(Debug)]
struct Record {
    wins: u32,
    losses: u32,
}

impl Default for Record {
    fn default() -> Self {
        Self { wins: 0, losses: 0 }
    }
}

fn standings(info: &[GameInfo]) -> HashMap<&String, Record> {
    let mut standing = HashMap::new();
    for game in info {
        let all_players = game.all_players();
        let winners = game.winners();
        for player in all_players {
            let record = standing.entry(player).or_insert_with(Record::default);
            if winners.contains(&player) {
                record.wins += 1;
            } else {
                record.losses += 1;
            }
        }
    }
    standing
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::File::open("example_games.yaml")?;
    let d: Vec<GameInfo> = serde_yaml::from_reader(f)?;
    println!("{:?}", d);
    println!("{:?}", standings(&d));
    Ok(())
}
