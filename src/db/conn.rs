use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use std::env;

pub fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let manager = ConnectionManager::<PgConnection>::new(url);

    Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("Failed to create pool")
}
