mod models;
mod database;
mod hiker_client;
mod handlers;
mod utils;
mod error;

use axum::{
    http::{HeaderValue, Method},
    routing::{get, post, put},
    Router,
};
use database::Database;
use hiker_client::HikerClient;
use std::env;
use tower_http::cors::CorsLayer;
use tower::ServiceBuilder;
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};
use tracing::info;
use std::sync::Arc;

use std::net::SocketAddr;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub hiker_client: HikerClient,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 로깅 초기화
    tracing_subscriber::fmt::init();

    // 환경 변수 로드
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    // 데이터베이스 연결 (자동 재연결 기능 포함)
    let db = Database::new(&database_url).await?;
    info!("Connected to database with auto-reconnection enabled");
    
    // Hiker API 클라이언트 초기화
    let hiker_client = HikerClient::new()?;
    info!("Hiker API client initialized");

    // 애플리케이션 상태
    let state = AppState {
        db,
        hiker_client,
    };

    // Rate limiting 설정 - IP당 초당 2개, 버스트 10개 허용
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)   // 초당 2개 요청 허용
            .burst_size(10)  // 순간적으로 10개까지 허용
            .finish()
            .unwrap(),
    );

    // CORS 설정 - 특정 도메인만 허용
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse::<HeaderValue>()?,
            "https://honestly-4xchoyd9u-luke-nams-projects.vercel.app/".parse::<HeaderValue>()?,  // 원하는 도메인으로 변경
            "honestly-80pcf0rqr-luke-nams-projects.vercel.app".parse::<HeaderValue>()?,  // www 버전도 추가
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::OPTIONS,  // preflight 요청을 위해 필요
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);  // 쿠키나 인증 정보를 포함한 요청 허용

    // 라우터 설정
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/:username", get(handlers::get_user))
        .route("/:username/negative/increment", put(handlers::increment_negative))
        .route("/:username/cool/increment", put(handlers::increment_cool))
        .route("/:username/negative/feedback", post(handlers::add_negative_feedback))
        .route("/:username/cool/feedback", post(handlers::add_cool_feedback))
        .layer(
            ServiceBuilder::new()
                // Rate limiting 적용
                .layer(GovernorLayer {
                    config: governor_conf,
                })
                // CORS 설정
                .layer(cors)
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("Server starting on http://0.0.0.0:3001");
    info!("Rate limiting enabled: 2 req/sec per IP, burst of 10");
    info!("CORS enabled for: localhost:3000 and your-domain.com");
    info!("");
    info!("Available endpoints:");
    info!("  GET  /health - Health check");
    info!("  GET  /:username - Get user info");
    info!("  PUT  /:username/negative/increment - Increment negative count");
    info!("  PUT  /:username/cool/increment - Increment cool count");
    info!("  POST /:username/negative/feedback - Add negative feedback with comment");
    info!("  POST /:username/cool/feedback - Add cool feedback with comment");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    
    Ok(())
}