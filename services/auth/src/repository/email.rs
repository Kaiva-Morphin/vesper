
use crate::{endpoints::login::LoginBody, AppState, CFG, ENV};
use message_broker::email::types::ChangedField;
use message_broker::email::types::Email;
use message_broker::email::types::EmailKind;
use rand::distr::Alphanumeric;
use rand::Rng;
use sea_orm::{prelude::Uuid, *};

use anyhow::Result;
use sha2::Digest;
use sha2::Sha256;
use shared::tokens::jwt::RefreshRules;
use shared::tokens::redis::Commands;
use shared::tokens::redis::RedisConn;
use shared::uuid;
use tracing::info;
use tracing::warn;

#[derive(Clone)]
pub struct EmailCode {
    code: String,
    email: String,
    kind: CodeKind
}

impl EmailCode {
    pub fn key_value(&self) -> (String, String) {
        // that looks weird
        match self.kind {
            CodeKind::PasswordRecovery => {
                (
                    Self::recovery_key(&self.code),
                    self.email.clone()
                )
            }
            CodeKind::Register => {
                (
                    self.email_key(),
                    self.code.clone()
                )
            }
        }
    }

    pub fn email_key(&self) -> String {
        format!("{}:{}", self.kind.to_prefix(), self.email)
    }

    pub fn recovery_key(code: &String) -> String {
        format!("{}:{}", CodeKind::PasswordRecovery.to_prefix(), format!("{:x}", Sha256::digest(code.as_bytes())))
    }

    pub fn register(email : String) -> Self {
        Self {
            code: rand::rng().random_range(100_000..1_000_000).to_string(),
            email,
            kind: CodeKind::Register
        }
    }

    pub fn recovery(email : String, code: String) -> Self {
        Self {
            code,
            email,
            kind: CodeKind::PasswordRecovery
        }
    }

    pub fn to_email(self) -> Email {
        Email{
            kind: match self.kind {
                CodeKind::Register => EmailKind::RegisterCode { code: self.code },
                CodeKind::PasswordRecovery => {
                    // todo!: link instead of code
                    EmailKind::RecoveryRequest { link: self.code }
                }
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

fn generate_reset_token() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(CFG.RECOVERY_TOKEN_LEN)
        .map(char::from)
        .collect()
}

trait RedisEmailCode {
    fn set_code(&self, value: EmailCode) -> Result<()>;
    fn set_recovery_code(&self, value: EmailCode) -> Result<()>;
    fn verify_code(&self, value: EmailCode) -> Result<bool>;
    fn pop_recovery_email(&self, code: &String) -> Result<Option<String>>;
}

impl RedisEmailCode for RedisConn {
    fn set_code(&self, email_code: EmailCode) -> Result<()> {
        if let CodeKind::PasswordRecovery = email_code.kind {return self.set_recovery_code(email_code)}
        let mut conn = self.pool.get()?;
        let (key, value) = email_code.key_value();
        let _ : () = conn.set_ex(key, value, email_code.kind.to_lifetime())?;
        Ok(())
    }

    fn set_recovery_code(&self, email_code: EmailCode) -> Result<()> {
        let mut conn = self.pool.get()?;
        let email_key = email_code.email_key();
        let (key, value) = email_code.key_value();

        // delete old key to ensure that recovery code is always one per email
        if let Some(v) = conn.get::<_, Option<String>>(&email_key)? {
            let _ : () = conn.del(v)?;
        }

        let _ : () = conn.set_ex(&key, &value, email_code.kind.to_lifetime())?;
        let _ : () = conn.set_ex(&email_key, &key, email_code.kind.to_lifetime())?;
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
    // code hash -> email
    fn pop_recovery_email(&self, code: &String) -> Result<Option<String>> {
        // bad: let (key , _) = EmailCode{kind: CodeKind::PasswordRecovery,code,email: "".to_owned(),}.key_value();
        let key = EmailCode::recovery_key(code); // much better
        let mut conn = self.pool.get()?;
        let r : Option<String> = conn.get(&key)?;
        let _  : () = conn.del(&key)?;
        Ok(r)
    }
}



impl AppState {
    pub async fn try_send_recovery_code(&self, email_or_login: &String) -> Result<()> {
        let Some(email) = self.get_email_from_login_cred(email_or_login).await? else {return Ok(())};
        let raw = generate_reset_token();
        info!("Generated recovery code");
        let code = EmailCode::recovery(email.clone(), raw);
        info!("Recovery code sent to nats!");
        let encoded =  bincode::encode_to_vec(&code.clone().to_email(), bincode::config::standard())?;
        self.publisher.publish(ENV.EMAIL_SEND_NATS_EVENT.clone(), encoded.into())
            .await?
            .await?;
        info!("Recovery code in redis!");
        self.redis.set_code(code)?;
        Ok(())
    }

    pub async fn send_changed_notification(&self, email: &String, field: ChangedField) -> Result<()> {
        let email = Email::changed(email.clone(), field);
        let encoded =  bincode::encode_to_vec(&email, bincode::config::standard())?;
        self.publisher.publish(ENV.EMAIL_SEND_NATS_EVENT.clone(), encoded.into())
            .await?
            .await?;
        info!("Changed notification sent to nats!");
        Ok(())
    }

    pub async fn send_register_code(&self, email: &String) -> Result<()> {
        let email = EmailCode::register(email.clone());
        info!("Generated code: {}", email.code);
        let encoded =  bincode::encode_to_vec(&email.clone().to_email(), bincode::config::standard())?;
        self.publisher.publish(ENV.EMAIL_SEND_NATS_EVENT.clone(), encoded.into())
            .await?
            .await?;
        info!("Register code sent to nats!");
        self.redis.set_code(email)?;
        info!("Register code in redis!");
        Ok(())
    }

    pub async fn send_new_login(&self, email: &String, ip : String, user_agent : String) -> Result<()> {
        info!("Sending new login to {}", email);
        let email = Email{
            to: email.clone(),
            kind: EmailKind::NewLogin { ip, user_agent }
        };
        let encoded =  bincode::encode_to_vec(&email, bincode::config::standard())?;
        self.publisher.publish(ENV.EMAIL_SEND_NATS_EVENT.clone(), encoded.into())
            .await?
            .await?;
        info!("Sended to nats!");
        Ok(())
    }

    pub async fn send_suspicious_refresh(&self, email: &String, ip : String, user_agent : String) -> Result<()> {
        info!("Sending suspicious refresh to {}", email);
        let email = Email{
            to: email.clone(),
            kind: EmailKind::SuspiciousRefresh { ip, user_agent }
        };
        let encoded =  bincode::encode_to_vec(&email, bincode::config::standard())?;
        self.publisher.publish(ENV.EMAIL_SEND_NATS_EVENT.clone(), encoded.into())
            .await?
            .await?;
        info!("Sended to nats!");
        Ok(())
    }

    

    pub async fn send_refresh_rules_update(&self, email: &String, ip : String, user_agent : String) -> Result<()> {
        info!("Sending rule update to {}", email);
        let email = Email{
            to: email.clone(),
            kind: EmailKind::RefreshRulesUpdate { ip, user_agent }
        };
        let encoded =  bincode::encode_to_vec(&email, bincode::config::standard())?;
        self.publisher.publish(ENV.EMAIL_SEND_NATS_EVENT.clone(), encoded.into())
            .await?
            .await?;
        info!("Sended to nats!");
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
    pub async fn recovery_password(&self, code: &String, new_password: String) -> Result<Option<()>> {
        if let Some(email) = self.redis.pop_recovery_email(code)? {
            self.set_password(&email, new_password).await?;
            self.send_changed_notification(&email, ChangedField::Password).await?;
            return Ok(Some(()));
        };
        Ok(None)
    }
}