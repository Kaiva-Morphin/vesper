use serde::{Deserialize, Serialize};



#[derive(Deserialize)]
pub struct AuthCallback {
    pub code: String
}



#[derive(Serialize, Deserialize, Clone)]
pub enum Service{
    Google,
    Discord
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub service: Service,
    pub id: String,
    pub avatar: String,
    pub email: String,
    pub verified: bool,
    pub name: String,
    pub nickname: String,
}



pub async fn does_exists(
    info: UserInfo,
){

}