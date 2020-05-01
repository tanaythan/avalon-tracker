use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

pub type Players = HashMap<String, Role>;

#[derive(Debug, Deserialize)]
pub struct GameInfo {
    pub players: Players,
    pub quests: Vec<Quest>,
    pub result: EndResult,
}

#[derive(Debug, Deserialize)]
pub struct Quest {
    pub status: QuestStatus,
    pub fails: Option<i32>,
    pub participants: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct EndResult {
    pub winner: Alignment,
    #[serde(rename = "type")]
    pub victory_type: VictoryType,
}

#[derive(Clone, Copy, Debug, Deserialize, sqlx::Type)]
#[sqlx(rename = "TEXT")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
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

impl std::convert::TryFrom<&str> for Role {
    type Error = sqlx::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use Role::*;
        Ok(match value {
            "assassin" => Assassin,
            "merlin" => Merlin,
            "minion" => Minion,
            "mordred" => Mordred,
            "morgana" => Morgana,
            "oberon" => Oberon,
            "percival" => Percival,
            "reverseoberon" => ReverseOberon,
            "servant" => Servant,

            _ => return Err(sqlx::Error::PoolClosed),
        })
    }
}

#[derive(Debug, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(rename = "TEXT")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum QuestStatus {
    Success,
    Fail,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, sqlx::Type)]
#[sqlx(rename = "TEXT")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Alignment {
    Good,
    Evil,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VictoryType {
    Assassination,
    Quest,
}

impl GameInfo {
    pub fn winners(&self) -> Vec<&String> {
        let alignment = self.result.winner;
        self.players
            .iter()
            .filter_map(|(k, v)| {
                if v.alignment() == alignment {
                    Some(k)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn all_players(&self) -> Vec<&String> {
        self.players.keys().collect()
    }

    pub fn players_with_alignment(&self, alignment: Alignment) -> Vec<&String> {
        self.players
            .iter()
            .filter_map(|(k, v)| {
                if v.alignment() == alignment {
                    Some(k)
                } else {
                    None
                }
            })
            .collect()
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

#[derive(PartialEq, Eq, Debug)]
pub struct Record {
    wins: u32,
    losses: u32,
}

impl Record {
    pub fn win_percentage(&self) -> f32 {
        let wins = self.wins as f32;
        let losses = self.losses as f32;
        wins / (wins + losses)
    }
}

impl Default for Record {
    fn default() -> Self {
        Self { wins: 0, losses: 0 }
    }
}

#[derive(Debug, Default)]
pub struct Standings<'a>(HashMap<&'a String, Record>);

impl<'a> fmt::Display for Standings<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::cmp::Ordering;
        writeln!(f, "Standings")?;
        writeln!(f, "{:^10} {:^4}   {:^4}  {:^4}", "Name", "W", "L", "%")?;

        let mut records: Vec<(&&'a String, &Record)> = self.0.iter().collect();
        records.sort_by(|(a_name, a), (b_name, b)| match b.wins.cmp(&a.wins) {
            #[allow(clippy::or_fun_call)]
            Ordering::Equal => b
                .win_percentage()
                .partial_cmp(&a.win_percentage())
                .unwrap_or(a_name.cmp(b_name)),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        });
        for player in records {
            writeln!(
                f,
                "{:<10} {:^4} - {:^4}: {:^4.2}",
                player.0,
                player.1.wins,
                player.1.losses,
                player.1.win_percentage()
            )?;
        }

        writeln!(f, "---------")
    }
}

pub fn standings(info: &[GameInfo]) -> Standings {
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
    Standings(standing)
}

pub fn standings_by_alignment(info: &[GameInfo]) -> HashMap<Alignment, Standings> {
    let mut standings = HashMap::new();
    for game in info {
        for alignment in &[Alignment::Good, Alignment::Evil] {
            let standing = standings
                .entry(*alignment)
                .or_insert_with(Standings::default);
            let players = game.players_with_alignment(*alignment);
            let winners = game.winners();
            for player in players {
                let record = standing.0.entry(player).or_insert_with(Record::default);
                if winners.contains(&player) {
                    record.wins += 1;
                } else {
                    record.losses += 1;
                }
            }
        }
    }
    standings
}

#[cfg(test)]
mod test {
    use super::*;
    const FILE: &str = "
- players:
    player1: merlin
    player2: morgana
    player3: percival
    player4: servant
    player5: assassin
  quests:
    - status: success
      fails: 0
      participants:
        - player1
        - player2
    - status: fail
      fails: 1
      participants:
        - player1
        - player2
        - player4
    - status: fail
      fails: 2
      participants:
        - player2
        - player4
        - player5
    - status: success
      fails: 0
      participants:
        - player1
        - player3
        - player4
    - status: success
      fails: 0
      participants:
        - player1
        - player3
        - player4

  result:
    winner: evil
    type: assassination

- players:
    player1: merlin
    player2: morgana
    player3: percival
    player4: servant
    player5: reverse_oberon
    player6: assassin
  quests:
    - status: success
      fails: 0
      participants:
        - player1
        - player2
    - status: fail
      fails: 1
      participants:
        - player1
        - player2
        - player4
    - status: fail
      fails: 2
      participants:
        - player2
        - player4
        - player5
    - status: success
      fails: 0
      participants:
        - player1
        - player3
        - player4
    - status: success
      fails: 0
      participants:
        - player1
        - player3
        - player4

  result:
    winner: good
    type: quest

";

    use std::collections::HashSet;
    use std::iter::FromIterator;
    #[test]
    fn test_read_from_test_file() {
        let games: Vec<GameInfo> = serde_yaml::from_str(FILE).unwrap();
        let d = games.get(0).unwrap();
        assert_eq!(
            d.winners().into_iter().collect::<HashSet<&String>>(),
            HashSet::from_iter(vec![&String::from("player2"), &String::from("player5")])
        );
        let standing = standings(&games);
        assert_eq!(
            standing.0.get(&String::from("player1")).unwrap(),
            &Record { wins: 1, losses: 1 }
        );
        assert_eq!(
            standing.0.get(&String::from("player5")).unwrap(),
            &Record { wins: 2, losses: 0 }
        );
    }
}
