// src/server.rs
use crate::{config::Config, redis::pool::RedisPoolManager};

#[derive(Clone)]
pub struct AppState {
    pub redis_pool: RedisPoolManager,
    pub config: Config,
}