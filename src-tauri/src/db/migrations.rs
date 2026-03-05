// IMPORTANT: Version numbers must be globally unique and monotonically increasing.
// NEVER reuse or insert between existing version numbers.
// Next available version: 8

use tauri_plugin_sql::{Migration, MigrationKind};

pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_games_table",
            sql: "CREATE TABLE IF NOT EXISTS games (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                name            TEXT NOT NULL,
                mod_dir         TEXT NOT NULL,
                staging_dir     TEXT NOT NULL,
                mod_structure   TEXT NOT NULL CHECK(mod_structure IN ('structured', 'loose')),
                requires_elevation INTEGER NOT NULL DEFAULT 0,
                created_at      INTEGER NOT NULL DEFAULT (unixepoch())
            );",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_mods_table",
            sql: "CREATE TABLE IF NOT EXISTS mods (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id         INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
                name            TEXT NOT NULL,
                enabled         INTEGER NOT NULL DEFAULT 0,
                staged_path     TEXT NOT NULL,
                imported_at     INTEGER NOT NULL DEFAULT (unixepoch())
            );",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "create_file_entries_table",
            sql: "CREATE TABLE IF NOT EXISTS file_entries (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                mod_id          INTEGER NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
                relative_path   TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_file_entries_mod_id ON file_entries(mod_id);
            CREATE INDEX IF NOT EXISTS idx_file_entries_path ON file_entries(relative_path, mod_id);",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "create_toggle_journal_table",
            sql: "CREATE TABLE IF NOT EXISTS toggle_journal (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                mod_id          INTEGER NOT NULL REFERENCES mods(id),
                operation       TEXT NOT NULL CHECK(operation IN ('enable', 'disable')),
                status          TEXT NOT NULL CHECK(status IN ('in_progress', 'done', 'rolled_back')),
                files_json      TEXT NOT NULL,
                started_at      INTEGER NOT NULL DEFAULT (unixepoch()),
                completed_at    INTEGER
            );",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "create_sub_mods_table",
            sql: "CREATE TABLE IF NOT EXISTS sub_mods (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                mod_id          INTEGER NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
                name            TEXT NOT NULL,
                folder_name     TEXT NOT NULL,
                enabled         INTEGER NOT NULL DEFAULT 0,
                user_enabled    INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_sub_mods_mod_id ON sub_mods(mod_id);",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "add_sub_mod_id_to_file_entries",
            sql: "ALTER TABLE file_entries ADD COLUMN sub_mod_id INTEGER REFERENCES sub_mods(id) ON DELETE CASCADE;
            CREATE INDEX IF NOT EXISTS idx_file_entries_sub_mod ON file_entries(sub_mod_id);",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 7,
            description: "create_profiles_and_profile_entries_tables",
            sql: "CREATE TABLE IF NOT EXISTS profiles (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id     INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
                name        TEXT NOT NULL,
                created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
                updated_at  INTEGER NOT NULL DEFAULT (unixepoch()),
                UNIQUE(game_id, name)
            );
            CREATE INDEX IF NOT EXISTS idx_profiles_game_id ON profiles(game_id);
            CREATE TABLE IF NOT EXISTS profile_entries (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id  INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
                mod_id      INTEGER NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
                enabled     INTEGER NOT NULL DEFAULT 0,
                sub_mod_states TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_profile_entries_profile ON profile_entries(profile_id);",
            kind: MigrationKind::Up,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_count_is_seven() {
        assert_eq!(get_migrations().len(), 7);
    }

    #[test]
    fn migration_versions_unique() {
        let migrations = get_migrations();
        let mut versions: Vec<i64> = migrations.iter().map(|m| m.version).collect();
        versions.sort();
        versions.dedup();
        assert_eq!(
            versions.len(),
            migrations.len(),
            "Migration versions must be unique"
        );
        // Verify monotonically increasing
        let raw: Vec<i64> = migrations.iter().map(|m| m.version).collect();
        let mut sorted = raw.clone();
        sorted.sort();
        assert_eq!(
            raw, sorted,
            "Migration versions must be monotonically increasing"
        );
    }
}
