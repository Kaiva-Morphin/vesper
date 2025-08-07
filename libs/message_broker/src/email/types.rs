use bincode::{Decode, Encode};



#[derive(Encode, Decode, Debug)]
pub struct Email {
    pub kind: EmailKind,
    pub to: String,
}

#[derive(Encode, Decode, Debug)]
pub enum EmailKind {
    RegisterCode {
        code: String
    },
    RecoveryRequest {
        link: String
    },
    ChangedNotification {
        changed_field: ChangedField
    },
    NewLogin {
        ip : String,
        user_agent : String,
    },
    SuspiciousRefresh {
        ip : String,
        user_agent : String,
    },
    RefreshRulesUpdate {
        ip : String,
        user_agent : String,
    }
}

impl EmailKind {
    pub fn name(&self) -> &'static str {
        match self {
            EmailKind::ChangedNotification { changed_field: _ } => "ChangedNotification",
            EmailKind::RegisterCode { code: _ } => "RegisterCode",
            EmailKind::RecoveryRequest { link: _ } => "RecoveryRequest",
            EmailKind::NewLogin { ip: _, user_agent: _ } => "NewLogin",
            EmailKind::SuspiciousRefresh { ip: _, user_agent: _ } => "SuspiciousRefresh",
            EmailKind::RefreshRulesUpdate { ip: _, user_agent: _ } => "RefreshRulesUpdate",
        }
    } 
}

#[derive(Encode, Decode, Debug)]
pub enum ChangedField {
    Password,
    Uid,
    Email
}

impl ChangedField {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Email => "Email",
            Self::Uid => "Uid",
            Self::Password => "Password"
        }
    }
}

impl Email {
    pub fn changed(email: String, field: ChangedField) -> Self {
        Self{
            kind: EmailKind::ChangedNotification {
                changed_field: field
            },
            to: email
        }
    }

}