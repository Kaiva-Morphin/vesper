use redis::Client;




#[derive(Clone)]
pub struct RedisConn{
    pub pool: r2d2::Pool<Client>
}
