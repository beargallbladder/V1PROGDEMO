use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::fs;
use std::path::Path;

pub async fn create_pool() -> anyhow::Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/stressor_leads".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    let migrations_dir = Path::new("migrations");

    if !migrations_dir.exists() {
        return Err(anyhow::anyhow!("Migrations directory not found"));
    }

    let mut migration_files: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().and_then(|s| s.to_str()) == Some("sql")
        })
        .collect();

    migration_files.sort_by_key(|entry| entry.path());

    for entry in migration_files {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();
        println!("Running migration: {}", filename);

        let sql = fs::read_to_string(&path)?;

        let statements: Vec<String> = sql
            .split(';')
            .map(|s| {
                s.lines()
                    .filter(|line| {
                        let trimmed = line.trim();
                        !trimmed.starts_with("--") && !trimmed.is_empty()
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        for (i, statement) in statements.iter().enumerate() {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(pool).await
                    .map_err(|e| anyhow::anyhow!("Error executing statement {}: {}\nStatement: {}", i + 1, e, trimmed))?;
            }
        }

        println!("Migration {} completed", filename);
    }

    Ok(())
}

