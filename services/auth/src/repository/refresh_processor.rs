use axum::{body::Body, http::{Response, StatusCode}, response::IntoResponse};
use axum_extra::extract::CookieJar;
use layers::logging::UserInfoExt;
use shared::{tokens::jwt::{RefreshRules, RefreshTokenPayload, RefreshTokenRecord, TokenEncoder}, utils::app_err::ToResponseBody};

use crate::{repository::{cookies::TokenCookie, tokens::{generate_access, generate_and_put_refresh}}, AppState};




pub struct RefreshProcessor<'a> {
    jar: CookieJar,
    state: &'a AppState,
    user_info: UserInfoExt,
    record: RefreshTokenRecord,
    refresh_payload: RefreshTokenPayload,
    refresh_rules: RefreshRules
}

impl<'a> RefreshProcessor<'a> {
    pub async fn begin(
        mut jar: CookieJar,
        state: &'a AppState,
        user_info: UserInfoExt,
    ) -> Result<Self, Response<Body>> {
        let Some(refresh_token_string) = jar.get_refresh() else {return Err((jar.rm_refresh(), StatusCode::UNAUTHORIZED).into_response())};
        jar = jar.rm_refresh();
        let Some(refresh_payload) = TokenEncoder::decode_refresh(refresh_token_string) else {return Err((jar, StatusCode::UNAUTHORIZED).into_response())};
        let record = state.redis_tokens.pop_refresh(refresh_payload.rtid).await.trough_app_err()?;
        let Some(record) = record else {return Err((jar.rm_refresh(), StatusCode::UNAUTHORIZED).into_response())};
        let refresh_rules = refresh_payload.rules.clone();
        Ok(RefreshProcessor {
            jar,
            state,
            user_info,
            record,
            refresh_payload,
            refresh_rules
        })
    }

    pub async fn check_refresh_rules(self) -> Result<Self, Response<Body>> {
        if !self.refresh_rules.allow_suspicious_refresh && (self.record.fingerprint != self.user_info.fingerprint || self.record.user_agent != self.user_info.user_agent) {
            self.state.send_suspicious_refresh(self.record.email.clone(), self.user_info.ip.clone(), self.user_info.user_agent.clone()).await.trough_app_err()?;
            if !self.refresh_rules.allow_suspicious_refresh {return Err((StatusCode::UNAUTHORIZED, self.jar, "Blocked due refresh rules").into_response());}
        }
        Ok(self)
    }

    pub async fn rm_all_refresh(self) -> Result<Self, Response<Body>> {
        self.state.redis_tokens.rm_all_refresh(self.refresh_payload.user).await.trough_app_err()?;
        Ok(self)
    }

    pub async fn update_refresh_rules(mut self, new_rules: RefreshRules) -> Result<Self, Response<Body>> {
        self.state.update_refresh_rules(self.record.email.clone(), self.user_info.ip.clone(), self.user_info.user_agent.clone(), &new_rules).await.trough_app_err()?;
        self.refresh_rules = new_rules;
        Ok(self)
    }


    pub async fn generate_tokens(self) -> Result<Response<Body>, Response<Body>> {
        let jar = generate_and_put_refresh(self.jar, self.state, &self.record.user, self.user_info, self.record.email, self.refresh_payload.rules).await.trough_app_err()?;
        let access_response = generate_access(self.record.user).trough_app_err()?;
        let v = (StatusCode::OK, jar, access_response).into_response();
        Ok(v)
    }
}