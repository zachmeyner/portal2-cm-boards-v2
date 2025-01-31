use crate::models::models::*;
use anyhow::Result;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

impl Demos {
    /// Gets Demo information for a given demo_id
    pub async fn get_demo(pool: &PgPool, demo_id: i64) -> Result<Option<Demos>> {
        let res = sqlx::query_as::<_, Demos>(r#"SELECT * FROM "p2boards".demos WHERE id = $1"#)
            .bind(demo_id)
            .fetch_one(pool)
            .await?;
        Ok(Some(res))
    }
    /// Returns a file id that can be used to download the demo
    #[allow(dead_code)]
    pub async fn get_demo_file_id(pool: &PgPool, demo_id: i64) -> Result<Option<String>> {
        let res = sqlx::query(r#"SELECT file_id FROM "p2boards".demos WHERE id = $1"#)
            .bind(demo_id)
            .map(|row: PgRow| row.get(0))
            .fetch_one(pool)
            .await?;
        Ok(Some(res))
    }
    /// Returns the partner's name
    #[allow(dead_code)]
    pub async fn get_partner_name(pool: &PgPool, demo_id: i64) -> Result<Option<String>> {
        let res = sqlx::query(r#"SELECT partner_name FROM "p2boards".demos WHERE id = $1"#)
            .bind(demo_id)
            .map(|row: PgRow| row.get(0))
            .fetch_one(pool)
            .await?;
        Ok(res)
    }
    /// Check to see if a demo was parsed successfully
    #[allow(dead_code)]
    pub async fn check_parsed(pool: &PgPool, demo_id: i64) -> Result<bool> {
        let res = sqlx::query(r#"SELECT parsed_successfully FROM "p2boards".demos WHERE id = $1"#)
            .bind(demo_id)
            .map(|row: PgRow| row.get(0))
            .fetch_one(pool)
            .await?;
        Ok(res)
    }
    /// Gets the SAR version associated with a demo
    #[allow(dead_code)]
    pub async fn get_sar_version(pool: &PgPool, demo_id: i64) -> Result<Option<String>> {
        let res: Option<String> =
            sqlx::query(r#"SELECT sar_version FROM "p2boards".demos WHERE id = $1"#)
                .bind(demo_id)
                .map(|row: PgRow| row.get(0))
                .fetch_one(pool)
                .await?;
        Ok(res)
    }
    /// Adds a new demo to the database, returns the demo's id
    pub async fn insert_demo(pool: &PgPool, demo: DemoInsert) -> Result<i64> {
        let mut res: i64 = 0;
        let _ = sqlx::query(
            r#"
                INSERT INTO "p2boards".demos 
               
                (file_id, partner_name, parsed_successfully, sar_version, cl_id) VALUES 
                ($1, $2, $3, $4, $5)
                RETURNING id"#,
        )
        .bind(demo.file_id)
        .bind(demo.partner_name)
        .bind(demo.parsed_successfully)
        .bind(demo.sar_version)
        .bind(demo.cl_id)
        .map(|row: PgRow| res = row.get(0))
        .fetch_one(pool)
        .await?;
        Ok(res)
    }
    /// Updates an existing demo
    #[allow(dead_code)]
    pub async fn update_demo(pool: &PgPool, updated_demo: Demos) -> Result<bool> {
        // TODO: Validation
        let _ = sqlx::query(
            r#"
                UPDATE "p2boards".demos
                SET file_id = $1, partner_name = $2, parsed_successfully = $3,
                sar_version = $4, cl_id = $5
                WHERE id = $6"#,
        )
        .bind(updated_demo.file_id)
        .bind(updated_demo.partner_name)
        .bind(updated_demo.parsed_successfully)
        .bind(updated_demo.sar_version)
        .bind(updated_demo.cl_id)
        .bind(updated_demo.id)
        .fetch_optional(pool)
        .await?;
        Ok(true)
    }
    /// Deletes a demo
    pub async fn delete_demo(pool: &PgPool, demo_id: i64) -> Result<bool> {
        let res = sqlx::query_as::<_, Demos>(
            r#"DELETE FROM "p2boards".demos 
                WHERE id = $1 RETURNING *"#,
        )
        .bind(demo_id)
        .fetch_one(pool)
        .await;
        match res {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("Error deleting demo -> {}", e);
                Ok(false)
            }
        }
    }
}
