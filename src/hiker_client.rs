use crate::models::{HikerApiResponse, User};
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::env;
use rand::Rng;

#[derive(Clone)]
pub struct HikerClient {
    client: Client,
    api_key: String,
}

impl HikerClient {
    pub fn new() -> Result<Self> {
        let api_key = env::var("HIKER_API_KEY")
            .map_err(|_| anyhow!("HIKER_API_KEY environment variable not found"))?;
        
        let client = Client::new();
        
        Ok(HikerClient { client, api_key })
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User> {
        let url = format!("https://api.hikerapi.com/v2/user/by/username?username={}", username);
        
        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .header("x-access-key", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("User not found in Hiker API"));
        }

        let hiker_response: HikerApiResponse = response.json().await?;

        let mut rng = rand::thread_rng();
        let init_cool_count = rng.gen_range(1..=50);
        let init_bad_count = rng.gen_range(1..=10);
        
        // Hiker API 응답을 우리 User 구조체로 변환
        let user = User {
            username: hiker_response.user.username,
            fullname: hiker_response.user.full_name.unwrap_or_else(|| "".to_string()),
            following_count: hiker_response.user.following_count.unwrap_or(0) ,
            follower_count: hiker_response.user.follower_count.unwrap_or(0),
            cool_comment: vec![], // 새 사용자는 빈 댓글로 시작
            cool_count: init_cool_count,
            bad_comment: vec![],
            bad_count: init_bad_count,
        };

        Ok(user)
    }
}
