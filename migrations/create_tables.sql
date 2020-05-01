CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    winner TEXT
);

CREATE TABLE IF NOT EXISTS player_roles (
    game_id TEXT,
    name TEXT,
    role TEXT
);

CREATE TABLE IF NOT EXISTS quests (
    id TEXT PRIMARY KEY,
    fails INTEGER,
    status TEXT
);

CREATE TABLE IF NOT EXISTS games_to_quests (
    game_id TEXT,
    quest_id TEXT,
    FOREIGN KEY(game_id) references games(id),
    FOREIGN KEY(quest_id) references quests(id)
);

CREATE TABLE IF NOT EXISTS quest_participants (
    quest_id TEXT,
    name TEXT,
    role TEXT,
    FOREIGN KEY(quest_id) REFERENCES quests(id)
);
