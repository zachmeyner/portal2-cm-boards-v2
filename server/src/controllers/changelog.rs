use anyhow::{Result, bail};
use std::collections::HashMap;
use sqlx::postgres::PgRow;
use sqlx::{Row, PgPool};
use chrono::NaiveDateTime;
use crate::models::models::*;

// Implementations of associated functions for Changelog
impl Changelog {
    pub async fn get_changelog(pool: &PgPool, cl_id: i64) -> Result<Option<Changelog>> {
        let res = sqlx::query_as::<_, Changelog>(r#"SELECT * FROM "p2boards".changelog WHERE id = $1"#)
            .bind(cl_id)
            .fetch_one(pool)
            .await?;
        Ok(Some(res))
    }
    #[allow(dead_code)]
    pub async fn get_demo_id_from_changelog(pool: &PgPool, cl_id: i64) -> Result<Option<i64>> {
        let res = sqlx::query(r#"SELECT demo_id FROM "p2boards".changelog WHERE id = $1"#)
            .bind(cl_id)
            .map(|row: PgRow| {row.get(0)})
            .fetch_one(pool)
            .await?;
        Ok(Some(res))
    }
    /// Check for if a given score already exists in the database, but is banned. Used for the auto-updating from Steam leaderboards.
    /// Returns `true` if there is a value found, `false` if no value, or returns an error.
    pub async fn check_banned_scores(pool: &PgPool, map_id: String, score: i32, profile_number: String, cat_id: i32) -> Result<bool> {
        // We don't care about the result, we only care if there is a result.
        let res = sqlx::query(r#" 
                SELECT * 
                FROM "p2boards".changelog
                WHERE changelog.score = $1
                AND changelog.map_id = $2
                AND changelog.profile_number = $3
                AND changelog.banned = $4
                AND changelog.category_id = $5"#)
            .bind(score)
            .bind(map_id)
            .bind(profile_number)
            .bind(true)
            .bind(cat_id)
            .fetch_optional(pool)
            .await?;
        match res {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
    /// Returns a vec of changelog for a user's PB history on a given SP map.
    pub async fn get_sp_pb_history(pool: &PgPool, profile_number: String, map_id: String) -> Result<Vec<Changelog>> {
        let res = sqlx::query_as::<_, Changelog>(r#" 
                SELECT * 
                FROM "p2boards".changelog
                WHERE changelog.profile_number = $1
                AND changelog.map_id = $2
                ORDER BY changelog.timestamp DESC NULLS LAST"#)
            .bind(profile_number)
            .bind(map_id)
            .fetch_all(pool)
            .await;
        match res{
            Ok(pb_history) => Ok(pb_history),
            Err(e) => Err(anyhow::Error::new(e).context("Could not find SP PB History")),
        }
    }
    /// Deletes all references to a demo_id in `changelog`
    pub async fn delete_references_to_demo(pool: &PgPool, demo_id: i64) -> Result<Vec<i64>> {
        let res: Vec<i64> = sqlx::query(r#"UPDATE "p2boards".changelog SET demo_id = NULL WHERE demo_id = $1 RETURNING id;"#)
            .bind(demo_id)
            .map(|row: PgRow| {row.get(0)})
            .fetch_all(pool)
            .await?;
        // eprintln!("{:#?}", res);
        Ok(res)
    }
    /// Deletes all references to a coop_id in `changelog`
    #[allow(dead_code)]
    pub async fn delete_references_to_coop_id(pool: &PgPool, coop_id: i64) -> Result<Vec<i64>> {
        let res: Vec<i64> = sqlx::query(r#"UPDATE "p2boards".changelog SET coop_id NULL WHERE coop_id = $1 RETURNING id;"#)
            .bind(coop_id)
            .map(|row: PgRow| {row.get(0)})
            .fetch_all(pool)
            .await?;
        Ok(res)
    }
    /// Insert a new changelog entry.
    pub async fn insert_changelog(pool: &PgPool, cl: ChangelogInsert) -> Result<i64> {
        // TODO: https://stackoverflow.com/questions/4448340/postgresql-duplicate-key-violates-unique-constraint
        let mut res: i64 = 0; 
        let _ = sqlx::query(r#"
                INSERT INTO "p2boards".changelog 
                (timestamp, profile_number, score, map_id, demo_id, banned, 
                youtube_id, coop_id, post_rank, pre_rank, submission, note,
                category_id, score_delta, verified, admin_note) VALUES 
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                RETURNING id"#)
            .bind(cl.timestamp).bind(cl.profile_number).bind(cl.score).bind(cl.map_id) // TODO: There has GOT to be a better way to do this... https://crates.io/crates/sqlxinsert ?
            .bind(cl.demo_id).bind(cl.banned).bind(cl.youtube_id).bind(cl.coop_id).bind(cl.post_rank)
            .bind(cl.pre_rank).bind(cl.submission).bind(cl.note).bind(cl.category_id)
            .bind(cl.score_delta).bind(cl.verified).bind(cl.admin_note)
            .map(|row: PgRow|{res = row.get(0)})
            .fetch_one(pool)
            .await?;
        Ok(res)
    }
    /// Updates all fields (except ID) for a given changelog entry. Returns the updated Changelog struct.
    pub async fn update_changelog(pool: &PgPool, update: Changelog) -> Result<bool> {
        let _ = sqlx::query(r#"UPDATE "p2boards".changelog 
                SET timestamp = $1, profile_number = $2, score = $3, map_id = $4, demo_id = $5, banned = $6, 
                youtube_id = $7, coop_id = $8, post_rank = $9, pre_rank = $10, submission = $11, note = $12,
                category_id = $13, score_delta = $14, verified = $15, admin_note = $16
                WHERE id = $17"#)
            .bind(update.timestamp).bind(update.profile_number).bind(update.score).bind(update.map_id) 
            .bind(update.demo_id).bind(update.banned).bind(update.youtube_id).bind(update.coop_id)
            .bind(update.post_rank).bind(update.pre_rank).bind(update.submission).bind(update.note)
            .bind(update.category_id).bind(update.score_delta).bind(update.verified).bind(update.admin_note)
            .bind(update.id)
            .fetch_optional(pool)
            .await?;
        Ok(true)
    }
    /// Updates demo_id
    pub async fn update_demo_id_in_changelog(pool: &PgPool, cl_id: i64, demo_id: i64) -> Result<bool> {
        let _ = sqlx::query(r#"UPDATE "p2boards".changelog 
                SET demo_id = $1 WHERE id = $2;"#)
            .bind(demo_id)
            .bind(cl_id)
            .fetch_optional(pool)
            .await?;
        Ok(true)
    }
    pub async fn delete_changelog(pool: &PgPool, cl_id: i64) -> Result<bool> {
        let res = sqlx::query_as::<_, Changelog>(r#"DELETE FROM "p2boards".changelog WHERE id = $1 RETURNING *"#)
            .bind(cl_id)
            .fetch_one(pool)
            .await;
        match res {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("Error deleting changelog -> {}", e);
                Ok(false)
            },
        }
    }  
}

impl ChangelogPage {
    /// Display page for the changelog
    ///
    /// Takes a list of parameters, returns a filtered list of changelog entries.
    ///
    /// Returns a [ChangelogPage], which contains information specifc for displaying on the web.
    pub async fn get_changelog_page(
        pool: &PgPool,
        params: ChangelogQueryParams,
    ) -> Result<Option<Vec<ChangelogPage>>> {
        // TODO: Add additonal filters
        
        let query_string = match build_filtered_changelog(pool, params, None).await {
            Ok(s) => s,
            Err(e) => bail!(e),
        };
        let res = sqlx::query_as::<_, ChangelogPage>(&query_string)
            .fetch_all(pool)
            .await;
        match res {
            Ok(changelog_filtered) => Ok(Some(changelog_filtered)),
            Err(e) => {
                eprintln!("{}", query_string);
                eprintln!("{}", e);
                Err(anyhow::Error::new(e).context("Error with SP Maps"))
            }
        }
    }
}

pub async fn build_filtered_changelog(pool: &PgPool, params: ChangelogQueryParams, additional_filters: Option<&mut Vec<String>>) -> Result<String> {
    let mut query_string: String = String::from(
        r#" 
        SELECT cl.id, cl.timestamp, cl.profile_number, cl.score, cl.map_id, cl.demo_id, cl.banned, 
        cl.youtube_id, cl.previous_id, cl.coop_id, cl.post_rank, cl.pre_rank, cl.submission, cl.note,
        cl.category_id, cl.score_delta, cl.verified, cl.admin_note, map.name AS map_name,  
        CASE
            WHEN u.board_name IS NULL
                THEN u.steam_name
            WHEN u.board_name IS NOT NULL
                THEN u.board_name
        END user_name, u.avatar
        FROM "p2boards".changelog AS cl
        INNER JOIN "p2boards".users AS u ON (u.profile_number = cl.profile_number)
        INNER JOIN "p2boards".maps AS map ON (map.steam_id = cl.map_id)
        INNER JOIN "p2boards".chapters AS chapter on (map.chapter_id = chapter.id)
    "#,
    );
    let mut filters: Vec<String> = Vec::new();
    if let Some(coop) = params.coop {
        if !coop {
            filters.push("chapter.is_multiplayer = False\n".to_string());
        } else if let Some(sp) = params.sp {
            if !sp {
                filters.push("chapter.is_multiplayer = True\n".to_string());
            }
        }
    }
    if let Some(has_demo) = params.has_demo {
        if has_demo {
            filters.push("cl.demo_id IS NOT NULL\n".to_string());
        } else {
            filters.push("cl.demo_id IS NULL\n".to_string());
        }
    }
    if let Some(yt) = params.yt {
        if yt {
            filters.push("cl.youtube_id IS NOT NULL\n".to_string());
        } else {
            filters.push("cl.youtube_id IS NULL\n".to_string());
        }
    }
    if let Some(wr_gain) = params.wr_gain {
        if wr_gain {
            filters.push("cl.post_rank = 1\n".to_string());
        }
    }
    if let Some(chamber) = params.chamber {
        filters.push(format!("cl.map_id = '{}'\n", &chamber));
    }
    if let Some(profile_number) = params.profile_number {
        filters.push(format!("cl.profile_number = {}\n", &profile_number));
    } else if let Some(nick_name) = params.nick_name {
        if let Some(profile_numbers) = Users::check_board_name(pool, nick_name.clone())
            .await?
            .as_mut()
        {
            if profile_numbers.len() == 1 {
                filters.push(format!(
                    "cl.profile_number = '{}'\n",
                    &profile_numbers[0].to_string()
                ));
            } else {
                let mut profile_str = format!(
                    "(cl.profile_number = '{}'\n",
                    &profile_numbers[0].to_string()
                );
                profile_numbers.remove(0);
                for num in profile_numbers.iter() {
                    profile_str.push_str(&format!(" OR cl.profile_number = '{}'\n", num));
                }
                profile_str.push(')');
                filters.push(profile_str);
            }
        } else {
            bail!("No users found with specified username pattern.");
        }
    }
    if let Some(first) = params.first {
        filters.push(format!("cl.id > {}\n", &first));
    } else if let Some(last) = params.last {
        filters.push(format!("cl.id < {}\n", &last));
    }
    if let Some(additional_filters) = additional_filters {
        filters.append(additional_filters);
    }
    // Build the statement based off the elements we added to our vector (used to make sure only first statement is WHERE, and additional are OR)
    for (i, entry) in filters.iter().enumerate() {
        if i == 0 {
            query_string = format!("{} WHERE {}", query_string, entry);
        } else {
            query_string = format!("{} AND {}", query_string, entry);
        }
    }
    //TODO: Maybe allow for custom order params????
    query_string = format!("{} ORDER BY cl.timestamp DESC NULLS LAST\n", query_string);
    if let Some(limit) = params.limit {
        query_string = format!("{} LIMIT {}\n", query_string, limit);
    } else {
        // Default limit
        query_string = format!("{} LIMIT 200\n", query_string);
    }
    Ok(query_string)
}

impl Default for ChangelogQueryParams {
    fn default() -> Self {
        ChangelogQueryParams {
            limit: Some(200),
            nick_name: None,
            profile_number: None,
            chamber: None,
            sp: Some(true),
            coop: Some(true),
            wr_gain: None,
            has_demo: None,
            yt: None,
            first: None,
            last: None,
        }
    }
}

impl ChangelogInsert {
    pub async fn new_from_submission(
        params: SubmissionChangelog,
        cache: HashMap<String, i32>,
    ) -> ChangelogInsert {
        ChangelogInsert {
            timestamp: match NaiveDateTime::parse_from_str(&params.timestamp, "%Y-%m-%d %H:%M:%S") {
                Ok(val) => Some(val),
                Err(_) => None,
            },
            profile_number: params.profile_number.clone(),
            score: params.score,
            map_id: params.map_id.clone(),
            youtube_id: params.youtube_id,
            note: params.note,
            category_id: params.category_id.unwrap_or(cache[&params.map_id]),
            submission: true,
            ..Default::default()
        }
    }
}