use crate::models::models::*;
use anyhow::Result;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

impl Users {
    /// Returns user information
    #[allow(dead_code)]
    pub async fn get_user(pool: &PgPool, profile_number: String) -> Result<Option<Users>> {
        let res = sqlx::query_as::<_, Users>(
            r#"SELECT * FROM "p2boards".users WHERE profile_number = $1"#,
        )
        .bind(profile_number)
        .fetch_one(pool)
        .await?;
        Ok(Some(res))
    }
    /// Gets a user's avatar and user_name/board_name (favors board_name)
    pub async fn get_user_data(pool: &PgPool, profile_number: String) -> Result<Option<UsersPage>> {
        let res = sqlx::query_as::<_, UsersPage>(
            r#"
                SELECT            
                CASE 
                    WHEN users.board_name IS NULL
                        THEN users.steam_name
                    WHEN users.board_name IS NOT NULL
                        THEN users.board_name
                    END user_name, users.avatar
                FROM "p2boards".users
                WHERE users.profile_number = $1
                "#,
        )
        .bind(profile_number)
        .fetch_optional(pool)
        .await?;
        Ok(res)
    }
    /// Pattern match on a given string to find similar names (supports board/steam names).
    pub async fn check_board_name(pool: &PgPool, nick_name: String) -> Result<Option<Vec<String>>> {
        let query_nn = format!("%{}%", &nick_name);
        let res = sqlx::query(
            r#"
                SELECT users.profile_number FROM "p2boards".users
                WHERE 
                    CASE
                        WHEN users.board_name IS NULL
                            THEN LOWER(users.steam_name) LIKE LOWER($1)
                        WHEN users.board_name IS NOT NULL
                            THEN LOWER(users.board_name) LIKE LOWER($1)
                    END
                "#,
        )
        .bind(query_nn)
        .map(|row: PgRow| row.get(0))
        .fetch_all(pool)
        .await?;
        if res.is_empty() {
            return Ok(None);
        }
        Ok(Some(res))
    }
    /// Returns a list of all banned player's profile_numbers.
    pub async fn get_banned(pool: &PgPool) -> Result<Vec<String>> {
        let res = sqlx::query(
            r#"SELECT users.profile_number FROM "p2boards".users WHERE users.banned = True"#,
        )
        .map(|row: PgRow| row.get(0))
        .fetch_all(pool)
        .await?;
        Ok(res)
    }
    /// Returns a list of all banned player's as a UsersDisplay object.
    pub async fn get_banned_display(pool: &PgPool) -> Result<Option<Vec<UsersDisplay>>> {
        let res = sqlx::query_as::<_, UsersDisplay>(
            r#" SELECT users.profile_number,
                COALESCE(users.board_name, users.steam_name) as user_name, 
                users.avatar
                    FROM "p2boards".users WHERE users.banned = 'true'"#,
        )
        .fetch_all(pool)
        .await?;
        Ok(Some(res))
    }
    /// Returns the boolean flag associated with the user in the boards, if Err, assumed User does not exist.
    pub async fn check_banned(pool: &PgPool, profile_number: String) -> Result<bool> {
        let res = sqlx::query(
            r#"SELECT users.banned FROM "p2boards".users WHERE users.profile_number = $1"#,
        )
        .bind(profile_number)
        .map(|row: PgRow| row.get(0))
        .fetch_one(pool)
        .await?;
        Ok(res)
    }
    /// Returns the title associated with the user (CAN BE NONE)
    #[allow(dead_code)]
    pub async fn get_title(pool: &PgPool, profile_number: String) -> Result<Option<String>> {
        // Result of query can be None, None is valid and should not return an error.
        let res: Option<String> =
            sqlx::query(r#"SELECT title FROM "p2boards".users WHERE users.profile_number = $1"#)
                .bind(profile_number)
                .map(|row: PgRow| row.get(0))
                .fetch_one(pool)
                .await?;
        Ok(res)
    }
    /// Returns the social media informatio associated with a given user's profile_number
    #[allow(dead_code)]
    pub async fn get_socials(pool: &PgPool, profile_number: String) -> Result<Option<Socials>> {
        let res = sqlx::query_as::<_, Socials>(
            r#"
                SELECT twitch, youtube, discord_id 
                FROM "p2boards".users 
                WHERE profile_number = $1"#,
        )
        .bind(profile_number)
        .fetch_one(pool)
        .await?;
        Ok(Some(res))
    }
    /// Returns the admin information associated with the user.
    #[allow(dead_code)]
    pub async fn get_admin_for_user(pool: &PgPool, profile_number: String) -> Result<Option<i32>> {
        let res = sqlx::query(r#"SELECT admin FROM "p2boards".users WHERE profile_number = $1"#)
            .bind(profile_number)
            .map(|row: PgRow| row.get(0))
            .fetch_one(pool)
            .await?;
        Ok(Some(res))
    }
    /// Returns UsersDisplay for all admins
    /// Usage:  admin_value = 0     -> Non-admin user
    ///         admin_value = 1     -> Standard admin
    ///         admin_value = 2     -> Shadow admin
    ///             (Has admin permissions, is not publically listed)
    ///             (Typically reserved for former admins, trusted players)
    ///         admin_value = 3     -> Developer admin
    ///             (Has admin permissions as an activen developer only)
    pub async fn get_all_admins(
        pool: &PgPool,
        admin_value: i32,
    ) -> Result<Option<Vec<UsersDisplay>>> {
        let res = sqlx::query_as::<_, UsersDisplay>(
            r#"
                SELECT users.profile_number, 
                    COALESCE(users.board_name, users.steam_name) AS user_name,
                    users.avatar
                FROM "p2boards".users
                WHERE users.admin = $1
                "#,
        )
        .bind(admin_value)
        .fetch_all(pool)
        .await?;
        Ok(Some(res))
    }
    /// Returns all users that have donated to the board. Ordered by highest amount.
    pub async fn get_donators(pool: &PgPool) -> Result<Option<Vec<Users>>> {
        let res = sqlx::query_as::<_, Users>(
            r#"
            SELECT * FROM "p2boards".users
                WHERE donation_amount IS NOT NULL
                ORDER BY CAST(donation_amount AS decimal) DESC;"#,
        )
        .fetch_all(pool)
        .await?;
        Ok(Some(res))
    }
    pub async fn get_profile(
        pool: &PgPool,
        profile_number: &String,
    ) -> Result<Option<ProfileData>> {
        let oldest = sqlx::query_as::<_, MapScoreDate>(r#"
            SELECT old.steam_id AS map, old.name AS map_name, old.score, old.timestamp FROM 
                (SELECT maps.steam_id, maps.name, changelog.score, changelog.timestamp FROM "p2boards".maps 
                INNER JOIN "p2boards".changelog ON (maps.steam_id = changelog.map_id) WHERE changelog.timestamp = (
                SELECT *
                    FROM (
                        SELECT MIN(o1.timestamp)
                        FROM
                        (SELECT DISTINCT ON (m1.steam_id) m1.steam_id, m1.name, cl1.score, cl1.timestamp, cl1.id
                            FROM "p2boards".changelog AS cl1
                            INNER JOIN "p2boards".maps AS m1
                                ON (cl1.map_id = m1.steam_id)
                            WHERE cl1.profile_number = $1
                            AND cl1.banned = 'false'
                            AND cl1.verified = 'true'
                            AND cl1.category_id = m1.default_cat_id
                            ORDER BY m1.steam_id, cl1.score) AS o1) AS a)) AS old;"#)
            .bind(profile_number)
            .fetch_one(pool)
            .await?;
        let newest = sqlx::query_as::<_, MapScoreDate>(r#"
            SELECT old.steam_id AS map, old.name AS map_name, old.score, old.timestamp FROM 
                (SELECT maps.steam_id, maps.name, changelog.score, changelog.timestamp FROM "p2boards".maps 
                INNER JOIN "p2boards".changelog ON (maps.steam_id = changelog.map_id) WHERE changelog.timestamp = (
                SELECT *
                    FROM (
                        SELECT MAX(o1.timestamp)
                        FROM
                        (SELECT DISTINCT ON (m1.steam_id) m1.steam_id, m1.name, cl1.score, cl1.timestamp, cl1.id
                            FROM "p2boards".changelog AS cl1
                            INNER JOIN "p2boards".maps AS m1
                                ON (cl1.map_id = m1.steam_id)
                            WHERE cl1.profile_number = $1
                            AND cl1.banned = 'false'
                            AND cl1.verified = 'true'
                            AND cl1.category_id = m1.default_cat_id
                            ORDER BY m1.steam_id, cl1.score) AS o1) AS a)) AS old;"#)
            .bind(profile_number)
            .fetch_one(pool)
            .await?;
        // let wrs = sqlx::query_as::<_, ProfileWrs>(r#""#)
        //     .bind(profile_number)
        //     .fetch_one(pool)
        //     .await?;
        Ok(Some(ProfileData { oldest, newest }))
    }
    // TODO: Consider using profanity filter (only for really bad names): https://docs.rs/censor/latest/censor/
    /// Inserts a new user into the databse
    pub async fn insert_new_users(pool: &PgPool, new_user: Users) -> Result<bool> {
        // let mut res = String::new();
        // We do not care about the returning profile_number. As it is not generated and we already have it
        let res = sqlx::query_as::<_, Users>(
            r#"
                INSERT INTO "p2boards".Users
                (profile_number, board_name, steam_name, banned, registered, 
                avatar, twitch, youtube, title, admin, donation_amount, discord_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                RETURNING *"#,
        )
        .bind(new_user.profile_number.clone())
        .bind(new_user.board_name)
        .bind(new_user.steam_name)
        .bind(new_user.banned)
        .bind(new_user.registered)
        .bind(new_user.avatar)
        .bind(new_user.twitch)
        .bind(new_user.youtube)
        .bind(new_user.title)
        .bind(new_user.admin)
        .bind(new_user.donation_amount)
        .bind(new_user.discord_id)
        .fetch_one(pool)
        .await?;

        if res.profile_number == new_user.profile_number {
            Ok(true)
        } else {
            Ok(false)
        }
    }
    #[allow(dead_code)]
    pub async fn update_existing_user(pool: &PgPool, updated_user: Users) -> Result<bool> {
        // If this gives us an error, we're updaing a user that already exists.
        let _ = Users::get_user(pool, updated_user.profile_number.clone()).await?;
        // TODO: Check to make sure user has correct AUTH to update specific items
        // (board_name should only be changed by the backend, admin should only be updated by admin etc)
        let _ = sqlx::query(
            r#"
                UPDATE "p2boards".Users
                SET board_name = $1, steam_name = $2, banned = $3, registered = $4, 
                avatar = $5, twitch = $6, youtube = $7, title = $8, admin = $9,
                donation_amount = $10, discord_id = $11
                WHERE profile_number = $12"#,
        )
        .bind(updated_user.board_name)
        .bind(updated_user.steam_name)
        .bind(updated_user.banned)
        .bind(updated_user.registered)
        .bind(updated_user.avatar)
        .bind(updated_user.twitch)
        .bind(updated_user.youtube)
        .bind(updated_user.title)
        .bind(updated_user.admin)
        .bind(updated_user.donation_amount)
        .bind(updated_user.discord_id)
        .bind(updated_user.profile_number)
        .fetch_optional(pool)
        .await?;
        Ok(true)
    }
    #[allow(dead_code)]
    pub async fn delete_user(pool: &PgPool, profile_number: String) -> Result<bool> {
        let res = sqlx::query_as::<_, Users>(
            r#"DELETE FROM "p2boards".users 
                WHERE profile_number = $1 RETURNING *"#,
        )
        .bind(profile_number)
        .fetch_one(pool)
        .await;
        match res {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("Error deleting user -> {}", e);
                Ok(false)
            }
        }
    }
}
