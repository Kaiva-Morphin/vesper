
use crate::{endpoints::login::LoginBody, AppState};
use redis_utils::{redis_tokens::RedisTokens, users::RedisUsers};
use rustperms::prelude::RustpermsDelta;
use rustperms_nodes::proto::WriteRequest;
use sea_orm::{prelude::Uuid, sea_query::Expr as QExpr, sqlx::Transaction, QuerySelect, *};

use anyhow::Result;

use postgre_entities::{user_data, user_mini_profile, user_profile};
use shared::{tokens::jwt::RefreshRules};
use tracing::{info, warn, error};


use bcrypt::{hash, DEFAULT_COST};

pub enum OauthLogin {
    EmailExists,
    Successful(Uuid, RefreshRules),
    NeedRegistration
} 


impl AppState {
    pub async fn is_uid_available(&self, uid: String) -> Result<bool> {
        let uid = uid.to_lowercase();
        let v = self.redis.get_user_guid(&uid).await?;
        Ok(v.is_none())
    }

    pub async fn login(&self, login_body: &LoginBody) -> Result<Option<(Uuid, RefreshRules)>> {
        let user = user_data::Entity::find()
            .filter(user_data::Column::Email.eq(&login_body.email))
            .one(&self.db).await?;
        let Some(user) = user else {return Ok(None)};
        if !bcrypt::verify(&login_body.password, &user.password)? {return Ok(None)}
        Ok(Some((user.guid, RefreshRules{warn_suspicious_refresh: user.warn_suspicious_refresh, allow_suspicious_refresh: user.allow_suspicious_refresh})))
    }
    
    pub async fn login_oauth(&self, provider: user_data::Column, id: &String, email: &String) -> Result<OauthLogin> {
        let users = user_data::Entity::find()
            .filter(user_data::Column::Email.eq(email.clone()))
            .column(user_data::Column::Guid)
            .column(user_data::Column::GoogleId)
            .column(user_data::Column::DiscordId)
            .column(user_data::Column::WarnSuspiciousRefresh)
            .column(user_data::Column::AllowSuspiciousRefresh)
            .expr_as(
                QExpr::col((user_data::Entity, provider))
                    .is_not_null()
                    .and(QExpr::col((user_data::Entity, provider)).ne(id.clone())),
                "email_exists",
            )
            .all(&self.db)
            .await?;
        if users.is_empty() {
            return Ok(OauthLogin::NeedRegistration);
        }
        let matching_user = users.iter().find(|u| u.google_id.as_ref() == Some(id));
        if let Some(user) = matching_user {
            return Ok(OauthLogin::Successful(
                user.guid.clone(),
                RefreshRules {
                    warn_suspicious_refresh: user.warn_suspicious_refresh,
                    allow_suspicious_refresh: user.allow_suspicious_refresh,
                },
            ));
        }
        return Ok(OauthLogin::EmailExists);
    }

    pub async fn delete_user(&self, email_or_uid: String, password: String) -> Result<bool> {
        let user = user_data::Entity::find()
            .filter(user_data::Column::Uid.eq(&email_or_uid).or(user_data::Column::Email.eq(&email_or_uid)))
            .one(&self.db).await?;
        let Some(user) = user else {return Ok(false)};
        if !bcrypt::verify(&password, &user.password)? {return Ok(false)}
        let d : RustpermsDelta = perms::user::delete_user(&user.guid).into();
        self.redis.remove_user(&user.guid, &user.uid).await.inspect_err(|e| error!("Failed to remove user from redis: {e}")).ok();
        self.redis.rm_all_refresh(&user.guid).await.inspect_err(|e| error!("Failed to remove refresh tokens from redis: {e}")).ok();
        user.delete(&self.db).await?;
        if let Ok(d) = d.serialize_to_string() {
            self.rustperms_master.clone().write_changes(WriteRequest{serialized_delta: d})
                .await
                .inspect_err(|e| error!("Failed to send rustperms delta for deleting user: {e}")).ok();
        } else {
            warn!("Failed to serialize rustperms delta for deleting user");
        }
        Ok(true)
    }

    pub async fn update_refresh_rules(&self, email : String, ip: String, user_agent: String,new_rules: &RefreshRules) -> Result<()> {
        info!("Updating refresh rules for {}", email);
        let user = user_data::Entity::find()
            .filter(user_data::Column::Email.eq(&email))
            .one(&self.db).await?;
        let Some(user) = user else {
            warn!("Can't find user!"); //? Is it possible?
            return Ok(())
        };
        let mut user: user_data::ActiveModel = user.into();
        user.allow_suspicious_refresh = Set(new_rules.allow_suspicious_refresh);
        user.warn_suspicious_refresh = Set(new_rules.warn_suspicious_refresh);
        user.update(&self.db).await?;
        if new_rules.is_insecure() {
            self.send_refresh_rules_update(email, ip, user_agent).await?;
        }
        Ok(())
    }

    pub async fn get_email_from_login_cred(
        &self,
        email_or_login: &String
    ) -> Result<Option<String>> {
        let user = user_data::Entity::find()
            .filter(user_data::Column::Uid.eq(email_or_login).or(user_data::Column::Email.eq(email_or_login)))
            .one(&self.db).await?;
        let Some(user) = user else {return Ok(None)};
        Ok(Some(user.email))
    }

    pub async fn set_password(
        &self,
        email: &String,
        new_password: String
    ) -> Result<()> {
        let user = user_data::Entity::find()
            .filter(user_data::Column::Email.eq(email))
            .one(&self.db).await?;
        let Some(user) = user else {
            warn!("Can't find user!"); //? Possible only if user request password recovery and delete account after?
            return Ok(())
        };
        info!("New password set for {}", user.guid);
        let mut user: user_data::ActiveModel = user.into();
        user.password = Set(bcrypt::hash(new_password, bcrypt::DEFAULT_COST)?);
        user.update(&self.db).await?;
        Ok(())
    }

    pub async fn register_user(
        &self,
        uid: String,
        nickname: String,
        email: String,
        password: String,
        google_id: Option<String>,
        discord_id: Option<String>,
    ) -> Result<Result<(Uuid, RefreshRules), String>> {
        let user_guid = Uuid::new_v4();
        let allow_suspicious_refresh = false;
        let warn_suspicious_refresh = true;
        let uid = uid.to_lowercase();

        let user = user_data::ActiveModel {
            guid: Set(user_guid),
            uid: Set(uid.clone()),
            nickname: Set(nickname),
            password: Set(hash(password, DEFAULT_COST)?),
            email: Set(email),
            allow_suspicious_refresh: Set(allow_suspicious_refresh),
            warn_suspicious_refresh: Set(warn_suspicious_refresh),
            google_id: Set(google_id),
            discord_id: Set(discord_id),
            ..Default::default()
        };
        let tsx = self.db.begin().await?;

        match user.insert(&tsx).await {
            Ok(_m) => {
                info!("Successful registration!");
                let mut u = perms::user::create_user(&user_guid);
                u.extend(perms::user::grant_default_for_user(&user_guid));
                let d : RustpermsDelta = u.into();
                if let Ok(d) = d.serialize_to_string() {
                    self.rustperms_master.clone().write_changes(WriteRequest{serialized_delta: d})
                        .await
                        .inspect_err(|e| error!("Failed to send rustperms delta for registering user: {e}")).ok();
                } else {
                    warn!("Failed to serialize rustperms delta for registering user");
                }

                let profile = user_profile::ActiveModel {
                    user_guid: Set(user_guid),
                    ..Default::default()
                };

                if let Err(e) = profile.insert(&tsx).await {
                    error!("Failed to create user profile: {e}");
                    tsx.rollback().await?;
                    return Ok(Err("Failed to create user profile".to_string()));
                }

                let miniprofile = user_mini_profile::ActiveModel {
                    user_guid: Set(user_guid),
                    ..Default::default()
                };

                if let Err(e) = miniprofile.insert(&tsx).await {
                    error!("Failed to create user miniprofile: {e}");
                    tsx.rollback().await?;
                    return Ok(Err("Failed to create user miniprofile".to_string()));
                }
                
                if let Err(e) = self.redis.add_user(&user_guid, &uid).await {
                    error!("Failed to set user guid in redis: {e}");
                    tsx.rollback().await?;
                    return Ok(Err("Failed to set user guid in redis".to_string()));
                }
                
                tsx.commit().await?;
                Ok(Ok((
                    user_guid,
                    RefreshRules{
                        warn_suspicious_refresh,
                        allow_suspicious_refresh,
                    }
                )))
            }
            Err(err) => {
                if let DbErr::Query(RuntimeErr::SqlxError(sqlx::error::Error::Database(err))) = &err {
                    let msg = err.message();
                    if msg.contains("duplicate key value") {
                        if msg.contains("email") {
                            info!("Conflict on email: Email already exists!");
                            return Ok(Err("The email is already registered. Please use a different email, or try to log in.".to_string()));
                        } else if msg.contains("uid") {
                            info!("Conflict on uid: User already exists!");
                            return Ok(Err("The uid is already taken. Please choose another, or try to log in.".to_string()));
                        }
                    }
                }
                error!("Uncatched error! {:#?}", err);
                tsx.rollback().await?;
                Err(anyhow::Error::from(err))
            }
        }
    }
}