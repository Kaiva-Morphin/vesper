pub async fn get_public_key() -> [u8; 451] {
    shared::tokens::jwt::get_public_key()
}