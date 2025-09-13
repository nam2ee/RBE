// handlers.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::{
    error::AppError,
    hiker_client::HikerClient,
    models::{ApiResponse, UserResponse},
    utils::get_random_comments,
    AppState,
};

#[derive(Deserialize)]
pub struct FeedbackRequest {
    pub comment: String,
}

pub async fn get_user(
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {

    if username == "nam_2ee.eth" || username =="_gyuuyg_"  {
        return 
        Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("그런 User는 없습니다".to_string()),
            }))
    }
    // 1. DB에서 사용자 찾기
    if let Some(user) = state.db.find_user(&username).await? {
        let random_cool_comments = get_random_comments(&user.cool_comment, 4);
        let random_bad_comments = get_random_comments(&user.bad_comment, 4);
        let total_comments = user.cool_comment.len() + user.bad_comment.len();
        let total_comments = total_comments as i32;

        let user_response = UserResponse {
            username: user.username,
            fullname: user.fullname,
            following_count: user.following_count,
            follower_count: user.follower_count,
            positive_votes: user.cool_count,
            negative_votes: user.bad_count,
            positive_comments: Some(random_cool_comments), // 필드명 변경
            negative_comments: Some(random_bad_comments),  // 필드명 변경
            total_comments: total_comments
        };
        return Ok(Json(ApiResponse {
            success: true,
            data: Some(user_response),
            message: None,
        }));
    }

    // 2. DB에 없음 - Hiker API 호출
    match state.hiker_client.get_user_by_username(&username).await {
        Ok(user) => {
            if let Err(e) = state.db.create_user(&user).await {
                eprintln!("Failed to save user to database: {}", e);
            }
            let user_response = UserResponse {
                username: user.username,
                fullname: user.fullname,
                following_count: user.following_count,
                follower_count: user.follower_count,
                positive_votes: user.cool_count,
                negative_votes: user.bad_count,
                positive_comments: Some(vec![]), // 빈 배열로 초기화
                negative_comments: Some(vec![]), // 빈 배열로 초기화
                total_comments: 0
            };


            Ok(Json(ApiResponse {
                success: true,
                data: Some(user_response),
                message: None,
            }))
        }
        Err(_) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("그런 User는 없습니다".to_string()),
            }))
        }
    }
}

// 나머지 핸들러들은 그대로 유지
async fn handle_user_creation(
    state: &AppState,
    username: &str,
) -> Result<bool, AppError> {
    if state.db.find_user(username).await?.is_none() {
        match state.hiker_client.get_user_by_username(username).await {
            Ok(user) => {
                state.db.create_user(&user).await?;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    } else {
        Ok(true)
    }
}

pub async fn increment_negative(
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    if !handle_user_creation(&state, &username).await? {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("그런 User는 없습니다".to_string()),
        }));
    }

    state.db.increment_negative_count(&username).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some("Negative count incremented".to_string()),
        message: None,
    }))
}

pub async fn increment_cool(
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    if !handle_user_creation(&state, &username).await? {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("그런 User는 없습니다".to_string()),
        }));
    }

    state.db.increment_cool_count(&username).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some("Cool count incremented".to_string()),
        message: None,
    }))
}

pub async fn add_negative_feedback(
    Path(username): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<FeedbackRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    if !handle_user_creation(&state, &username).await? {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("그런 User는 없습니다".to_string()),
        }));
    }

    state.db.add_negative_feedback(&username, &payload.comment).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some("Negative feedback added successfully".to_string()),
        message: None,
    }))
}

pub async fn add_cool_feedback(
    Path(username): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<FeedbackRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    if !handle_user_creation(&state, &username).await? {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("그런 User는 없습니다".to_string()),
        }));
    }

    state.db.add_cool_feedback(&username, &payload.comment).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some("Cool feedback added successfully".to_string()),
        message: None,
    }))
}

pub async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse {
        success: true,
        data: Some("Backend is healthy".to_string()),
        message: None,
    })
}
