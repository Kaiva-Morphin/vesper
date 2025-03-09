
use crate::{endpoints::login::LoginBody, AppState};
use sea_orm::{prelude::Uuid, *};

use anyhow::Result;

use postgre_entities::user_data;
use shared::{default_err, tokens::jwt::RefreshRules};
use tracing::{info, warn};

use crate::endpoints::register::RegisterBody;

use bcrypt::{hash, DEFAULT_COST};



impl AppState {
    pub async fn is_login_available(&self, login: String) -> Result<bool> {
        let v = user_data::Entity::find()
            .filter(user_data::Column::Login.eq(login))
            .one(&self.db).await;
        Ok(v?.is_none())
    }

    pub async fn login(&self, login_body: &LoginBody) -> Result<Option<(Uuid, RefreshRules)>> {
        let user = user_data::Entity::find()
            .filter(user_data::Column::Login.eq(&login_body.email_or_login).or(user_data::Column::Email.eq(&login_body.email_or_login)))
            .one(&self.db).await?;
        let Some(user) = user else {return Ok(None)};
        if !bcrypt::verify(&login_body.password, &user.password)? {return Ok(None)}
        Ok(Some((user.uuid, RefreshRules{warn_suspicious_refresh: user.warn_suspicious_refresh, allow_suspicious_refresh: user.allow_suspicious_refresh})))
    }

    pub async fn update_refresh_rules(&self, email : &String, ip: String, user_agent: String,new_rules: RefreshRules) -> Result<()> {
        info!("Updating refresh rules for {}", email);
        let user = user_data::Entity::find()
            .filter(user_data::Column::Email.eq(email))
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
            .filter(user_data::Column::Login.eq(email_or_login).or(user_data::Column::Email.eq(email_or_login)))
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
            warn!("Can't find user!"); //? Possible only if user request password recovery and delete account?
            return Ok(())
        };
        info!("New password set for {}",  user.uuid);
        let mut user: user_data::ActiveModel = user.into();
        user.password = Set(bcrypt::hash(new_password, bcrypt::DEFAULT_COST)?);
        user.update(&self.db).await?;
        Ok(())
    }


    pub async fn register_user(
        &self,
        register_body: RegisterBody
    ) -> Result<Result<(Uuid, RefreshRules), String>> {
        let user_uuid = Uuid::new_v4();
        let allow_suspicious_refresh = false;
        let warn_suspicious_refresh = true;
        let user = user_data::ActiveModel {
            uuid: Set(user_uuid.clone()),
            login: Set(register_body.login),
            nickname: Set(register_body.nickname),
            password: Set(hash(register_body.password, DEFAULT_COST)?),
            email: Set(register_body.email),
            allow_suspicious_refresh: Set(allow_suspicious_refresh),
            warn_suspicious_refresh: Set(warn_suspicious_refresh),
            ..Default::default()
        };
        match user.insert(&self.db).await {
            Ok(_m) => {
                info!("Successful registration!");
                return Ok(Ok((
                    user_uuid,
                    RefreshRules{
                        warn_suspicious_refresh,
                        allow_suspicious_refresh,
                    }
                )))
            }
            Err(DbErr::Query(RuntimeErr::SqlxError(sqlx::error::Error::Database(err)))) 
            if err.message().contains("duplicate key value") => {
                if err.message().contains("email") {
                    info!("Conflict on email: Email already exists!");
                    return Ok(Err("The email is already registered. Please use a different email, or try to log in.".to_string()));
                } else if err.message().contains("login") {
                    info!("Conflict on login: User already exists!");
                    return Ok(Err("The login is already taken. Please choose another, or try to log in.".to_string()));
                }
                warn!("Uncatched error! {:#?}", err);
            }
            Err(err) => {
                warn!("Uncatched error! {:#?}", err);
            }
        }
        default_err!()
    }
}