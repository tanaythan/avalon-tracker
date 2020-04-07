use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

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
#[serde(rename_all = "snake_case")]
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
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
            .filter_map(|(k, v)| match v.alignment() == alignment {
                true => Some(k),
                false => None,
            })
            .collect()
    }

    pub fn all_players(&self) -> Vec<&String> {
        self.players.keys().collect()
    }

    pub fn players_with_alignment(&self, alignment: Alignment) -> Vec<&String> {
        self.players
            .iter()
            .filter_map(|(k, v)| match v.alignment() == alignment {
                true => Some(k),
                false => None,
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
struct Record {
    wins: u32,
    losses: u32,
}

impl Default for Record {
    fn default() -> Self {
        Self { wins: 0, losses: 0 }
    }
}

#[derive(Debug, Default)]
struct Standings<'a>(HashMap<&'a String, Record>);

impl<'a> fmt::Display for Standings<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Standings")?;

        let mut records: Vec<(&&'a String, &Record)> = self.0.iter().collect();
        records.sort_by(|a, b| b.1.wins.cmp(&a.1.wins));
        for player in records {
            writeln!(f, "{}: {} - {}", player.0, player.1.wins, player.1.losses)?;
        }

        writeln!(f, "---------")
    }
}

fn standings(info: &[GameInfo]) -> Standings {
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

fn standings_by_alignment(info: &[GameInfo]) -> HashMap<Alignment, Standings> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::File::open("example_games.yaml")?;
    let d: Vec<GameInfo> = serde_yaml::from_reader(f)?;
    println!("{:?}", d);
    println!("{}", standings(&d));
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const FILE: &'static str = "
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
