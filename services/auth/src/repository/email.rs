
use crate::{endpoints::login::LoginBody, AppState, CFG, ENV};
use message_broker::email::types::Email;
use message_broker::email::types::EmailKind;
use rand::Rng;
use sea_orm::{prelude::Uuid, *};

use anyhow::Result;
use shared::tokens::redis::Commands;
use shared::tokens::redis::RedisConn;
use shared::uuid;
use tracing::info;

#[derive(Clone)]
pub struct EmailCode {
    code: String,
    email: String,
    kind: CodeKind
}

impl EmailCode {
    pub fn key_value(&self) -> (String, String) {
        (
            format!("{}:{}", self.kind.to_prefix(), self.email),
            self.code.clone()
        )
    }

    pub fn register(email : String) -> Self {
        Self {
            code: rand::rng().random_range(100_000..1_000_000).to_string(),
            email,
            kind: CodeKind::Register
        }
    }

    pub fn recovery(email : String) -> Self {
        Self {
            code: uuid::Uuid::new_v4().to_string(),
            email,
            kind: CodeKind::Register
        }
    }

    pub fn to_email(self) -> Email {
        Email{
            kind: match self.kind {
                CodeKind::Register => EmailKind::RegisterCode { code: self.code },
                CodeKind::PasswordRecovery => EmailKind::RecoveryRequest { link: unimplemented!() /*self.code*/ }
            },
            to: self.email
        }
    }
}

#[derive(Clone)]
pub enum CodeKind {
    Register,
    PasswordRecovery
}

impl CodeKind {
    pub fn to_prefix(&self) -> &'static str {
        match self {
            CodeKind::Register => "REGC",
            CodeKind::PasswordRecovery => "RECC"
        }
    }

    pub fn to_lifetime(&self) -> u64{
        match self {
            CodeKind::PasswordRecovery => CFG.RECOVERY_EMAIL_LIFETIME,
            CodeKind::Register => CFG.REGISTER_EMAIL_LIFETIME
        }
    }
}

trait RedisEmailCode {
    fn set_code(&self, value: EmailCode) -> Result<()>;
    fn verify_code(&self, value: EmailCode) -> Result<bool>;
}

impl RedisEmailCode for RedisConn {
    fn set_code(&self, email_code: EmailCode) -> Result<()> {
        let (key , value) = email_code.key_value();
        let mut conn = self.pool.get()?;
        let _ : () = conn.set_ex(key, value, email_code.kind.to_lifetime())?;
        Ok(())
    }
    fn verify_code(&self, email_code: EmailCode) -> Result<bool> {
        let (key , value) = email_code.key_value();
        let mut conn = self.pool.get()?;
        let r : Option<String> = conn.get(key)?;
        if let Some(record) = r {
            return Ok(record == value)
        }
        Ok(false)
    }
}



impl AppState {
    pub async fn send_register_code(&self, email: &String) -> Result<()> {
        let code = EmailCode::register(email.clone());
        let encoded =  bincode::encode_to_vec(&code.clone().to_email(), bincode::config::standard())?;
        self.publisher.publish(ENV.EMAIL_SEND_NATS_EVENT.clone(), encoded.into())
            .await?
            .await?;
        info!("Register code sent to nats!");
        info!("Generated code: {}", code.code);
        self.redis.set_code(code)?;
        info!("Register code in redis!");
        Ok(())
    }
    pub fn verify_register_code(&self, code: String, email: String) -> Result<bool> {
        let email_code = EmailCode{
            kind: CodeKind::Register,
            code,
            email
        };
        Ok(self.redis.verify_code(email_code)?)
    }
}