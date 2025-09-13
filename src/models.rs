use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub fullname: String,
    pub following_count: i32,  // u32 → i32
    pub follower_count: i32,   // u32 → i32
    pub cool_comment: Vec<String>,
    pub cool_count: i32,       // u32 → i32
    pub bad_comment: Vec<String>,
    pub bad_count: i32,        // u32 → i32
}

#[derive(Debug, Deserialize)]
pub struct HikerApiResponse {
    pub user: HikerUser,
}

#[derive(Debug, Deserialize)]
pub struct HikerUser {
    pub username: String,
    pub full_name: Option<String>,
    pub follower_count: Option<i32>,  // u32 → i32
    pub following_count: Option<i32>, // u32 → i32
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub username: String,
    pub fullname: String,
    pub following_count: i32,  // u32 → i32
    pub follower_count: i32,   // u32 → i32
    pub positive_votes: i32,   // u32 → i32
    pub negative_votes: i32,   // u32 → i32
    pub positive_comments: Option<Vec<String>>,
    pub negative_comments: Option<Vec<String>>,
    pub total_comments: i32
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}