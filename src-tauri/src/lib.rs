pub mod commands;
pub mod db;
pub mod error;
pub mod services;
pub mod state;

use specta_typescript::{BigIntExportBehavior, Typescript};
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::games::add_game,
        commands::games::remove_game,
        commands::games::edit_game,
        commands::games::list_games,
        commands::integrity::run_integrity_scan,
        commands::mods::import_mod,
        commands::mods::list_mods,
        commands::mods::list_mod_files,
        commands::mods::list_sub_mods_cmd,
        commands::mods::toggle_mod_cmd,
        commands::mods::toggle_sub_mod_cmd,
        commands::mods::delete_mod_cmd,
        commands::mods::check_conflicts_cmd,
    ]);

    #[cfg(debug_assertions)]
    {
        let bindings_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts");
        builder
            .export(
                Typescript::default().bigint(BigIntExportBehavior::Number),
                &bindings_path,
            )
            .expect("Failed to export typescript bindings");
    }

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:modtoggler.db", db::migrations::get_migrations())
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(state::AppState::default())
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            // Create a sqlx::SqlitePool pointing to the same DB file that tauri-plugin-sql manages.
            // This gives Rust commands direct access to the database via sqlx queries.
            let app_config_dir = app
                .path()
                .app_config_dir()
                .expect("No app config dir found");
            std::fs::create_dir_all(&app_config_dir)
                .expect("Failed to create app config dir");
            let db_path = app_config_dir.join("modtoggler.db");
            let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

            let pool = tauri::async_runtime::block_on(async {
                let pool = sqlx::SqlitePool::connect(&db_url)
                    .await
                    .expect("Failed to connect to SQLite database");

                // Run migrations directly on the sqlx pool so Rust commands always have tables.
                // This mirrors the DDL from db::migrations but runs on the pool Rust commands use.
                // Using raw_sql to support multi-statement migrations (e.g., CREATE TABLE + CREATE INDEX).
                for migration in db::migrations::get_migrations() {
                    match sqlx::raw_sql(migration.sql).execute(&pool).await {
                        Ok(_) => {}
                        Err(e) => {
                            let msg = e.to_string();
                            // ALTER TABLE ADD COLUMN is not idempotent in SQLite —
                            // ignore "duplicate column" errors on subsequent startups.
                            if !msg.contains("duplicate column name") {
                                panic!("Failed to run migration {}: {}", migration.description, e);
                            }
                        }
                    }
                }

                pool
            });

            app.manage(pool);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
