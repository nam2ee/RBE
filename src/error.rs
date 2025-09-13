use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// 애플리케이션의 모든 에러를 위한 커스텀 에러 타입
pub struct AppError(anyhow::Error);

// AppError가 HTTP 응답으로 변환되는 방법을 정의
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 내부 에러를 로깅합니다 (실제 프로덕션에서는 tracing/log 라이브러리 사용 권장).
        eprintln!("Internal server error: {:?}", self.0);

        let body = Json(json!({
            "success": false,
            "data": null,
            "message": "서버 내부 오류가 발생했습니다.",
        }));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

// anyhow::Error에서 AppError로 변환할 수 있도록 `From` 트레이트를 구현합니다.
// 이를 통해 핸들러에서 `?` 연산자를 자유롭게 사용할 수 있습니다.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
