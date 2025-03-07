
use crate::{endpoints::login::LoginBody, AppState};
use sea_orm::{prelude::Uuid, *};

use anyhow::Result;

use postgre_entities::user_data;
use shared::default_err;
use tracing::{info, warn};

use crate::endpoints::register::RegisterBody;

use bcrypt::{hash, DEFAULT_COST};


pub struct ErrOption<T>(Option<T>);

impl<T> ErrOption<T> {
    pub fn err(self) -> Option<T> {
        self.0
    }
}

impl<F> From<Option<F>> for ErrOption<F> {
    fn from(value: Option<F>) -> Self {
        ErrOption(value)
    }
}


impl AppState {
    pub async fn is_login_available(&self, login: String) -> Result<bool> {
        let v = user_data::Entity::find()
            .filter(user_data::Column::Login.eq(login))
            .one(&self.db).await;
        Ok(v?.is_none())
    }

    pub async fn login(&self, login_body: &LoginBody) -> Result<Option<Uuid>> {
        let user = user_data::Entity::find()
            .filter(user_data::Column::Login.eq(&login_body.login).or(user_data::Column::Email.eq(&login_body.login)))
            .one(&self.db).await?;
        let Some(user) = user else {return Ok(None)};
        if !bcrypt::verify(&login_body.password, &user.password)? {return Ok(None)}
        Ok(Some(user.uuid))
    }


    pub async fn register_user(
        &self,
        register_body: RegisterBody
    ) -> Result<Result<Uuid, String>> {
        let user_uuid = Uuid::new_v4();
        let user = user_data::ActiveModel {
            uuid: Set(user_uuid.clone()),
            login: Set(register_body.login),
            nickname: Set(register_body.nickname),
            password: Set(hash(register_body.password, DEFAULT_COST)?),
            email: Set(register_body.email),
            ..Default::default()
        };
        match user.insert(&self.db).await {
            Ok(_m) => {info!("Successful registration!"); return Ok(Ok(user_uuid))}
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