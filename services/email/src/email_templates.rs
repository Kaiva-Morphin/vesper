use lettre::Message;
use lettre::message::{header, Mailbox, SinglePart};

use anyhow::Result;
use message_broker::email::types::EmailKind;

use crate::ENV;

pub trait ToMessage {
    fn to_message(self) -> Result<Message>;
}

impl ToMessage for message_broker::email::types::Email {
    fn to_message(self) -> Result<Message> {
        let email = self.to.parse()?;
        let default_message = Message::builder()
            .to(email)
            .from(Mailbox::new(Some("Vesper".to_owned()), ENV.EMAIL_SENDER.clone()));
        let default_singlepart = SinglePart::builder().header(header::ContentType::TEXT_PLAIN);
        Ok(match self.kind {
            // todo!: prettify
            EmailKind::RegisterCode {code} => default_message.subject("Register code").singlepart(default_singlepart.body(code))?,
            EmailKind::RecoveryRequest { link } => default_message.subject("Password recovery").singlepart(default_singlepart.body(link))?,
            EmailKind::ChangedNotification { changed_field } => default_message.subject(format!("{} changed", changed_field.to_str())).singlepart(default_singlepart.body(changed_field.to_str().to_string()))?,
            EmailKind::NewLogin {ip, user_agent} => default_message.subject("New login").singlepart(default_singlepart.body(format!("Ip: {ip} UserAgent: {user_agent}")))?,
            EmailKind::SuspiciousRefresh {ip, user_agent} => default_message.subject("Suspicious refresh").singlepart(default_singlepart.body(format!("Ip: {ip} UserAgent: {user_agent}")))?,
            EmailKind::RefreshRulesUpdate {ip, user_agent} => default_message.subject("Refresh rules update").singlepart(default_singlepart.body(format!("Someone updated yours refresh rules\nIp: {ip} UserAgent: {user_agent}")))?,
        })
        
    }
}