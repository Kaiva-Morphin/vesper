use std::str::FromStr;
use cookie::CookieJar;







pub trait ExtraCookie {
    fn get_typed<T: FromStr>(&self, name: &str) -> Option<T>;
}
impl ExtraCookie for CookieJar {
    fn get_typed<T: FromStr>(&self, name: &str) -> Option<T> {
        self.get(name).and_then(|cookie| cookie.value().parse::<T>().ok())
    }
    // fn put_user(self, user: AnonymousUser) -> Self {
    //     self.add(Cookie::new(COOKIE_PUBLIC_ID, user.public_id.to_string()))
    //         .add(Cookie::new(COOKIE_PRIVATE_ID, user.id.to_string()))
    // }
    // fn get_passwd_hash(&self) -> Option<String> {
    //     self.get(COOKIE_PASSWORD_HASH).map(|v| v.value().to_string())
    // }
    // fn get_user_id(&self) -> Option<UserId> {
    //     self.get_typed(COOKIE_PRIVATE_ID)
    // }
    // fn get_public_user_id(&self) -> Option<PublicUserId> {
    //     self.get_typed(COOKIE_PUBLIC_ID)
    // }
    // fn get_room_id(&self) -> Option<RoomId> {
    //     self.get_typed(COOKIE_ROOM_ID)
    // }
}