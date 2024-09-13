use crate::utils::rand;
use anyhow::Context;
use spring_redis::{redis::AsyncCommands, Redis};
use spring_web::error::Result;

pub async fn get_validate_code(redis: &mut Redis, email: &str) -> Result<Option<String>> {
    let key = validate_redis_key(email);
    Ok(redis
        .get(&key)
        .await
        .with_context(|| format!("get {} from redis failed", key))?)
}

pub async fn gen_validate_code(redis: &mut Redis, email: &str) -> Result<String>{
    gen_validate_code_with_duration(redis, email, 5 * 60).await
}

pub async fn gen_validate_code_with_duration(
    redis: &mut Redis,
    email: &str,
    seconds: u64,
) -> Result<String> {
    let key = validate_redis_key(email);
    let rand_code = rand::rand_alphanumeric(6);
    redis
        .set_ex(&key, &rand_code, seconds)
        .await
        .with_context(|| format!("set {} to redis failed", key))?;
    Ok(rand_code)
}

fn validate_redis_key(email: &str) -> String {
    format!("email-validate:{email}")
}