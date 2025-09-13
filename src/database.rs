use tokio_postgres::Client;
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use crate::models::User;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

#[derive(Clone)]
pub struct Database {
    client: Arc<RwLock<Arc<Client>>>,
    database_url: String,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let client = Self::create_connection(database_url).await?;
        
        let db = Database {
            client: Arc::new(RwLock::new(Arc::new(client))),
            database_url: database_url.to_string(),
        };
        
        db.init_tables().await?;
        
        // 연결 상태 모니터링 태스크 시작
        db.start_connection_monitor();
        
        Ok(db)
    }
    
    async fn create_connection(database_url: &str) -> Result<Client> {
        let builder = SslConnector::builder(SslMethod::tls())?;
        let connector = MakeTlsConnector::new(builder.build());
        
        let (client, connection) = tokio_postgres::connect(database_url, connector).await?;
        
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });
        
        Ok(client)
    }
    
    fn start_connection_monitor(&self) {
        let client = self.client.clone();
        let database_url = self.database_url.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // 30초마다 체크
            
            loop {
                interval.tick().await;
                
                let is_closed = {
                    let client_guard = client.read().await;
                    client_guard.is_closed()
                };
                
                if is_closed {
                    eprintln!("Database connection lost, attempting to reconnect...");
                    
                    // 재연결 시도 (최대 5회)
                    for attempt in 1..=5 {
                        match Self::create_connection(&database_url).await {
                            Ok(new_client) => {
                                let mut client_guard = client.write().await;
                                *client_guard = Arc::new(new_client);
                                eprintln!("Database reconnected successfully");
                                break;
                            }
                            Err(e) => {
                                eprintln!("Reconnection attempt {} failed: {}", attempt, e);
                                if attempt < 5 {
                                    tokio::time::sleep(Duration::from_secs(2 * attempt)).await;
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    
    async fn get_client(&self) -> Arc<Client> {
        self.client.read().await.clone()
    }
    
    async fn ensure_connection(&self) -> Result<()> {
        let is_closed = {
            let client = self.get_client().await;
            client.is_closed()
        };
        
        if is_closed {
            let new_client = Self::create_connection(&self.database_url).await?;
            let mut client_guard = self.client.write().await;
            *client_guard = Arc::new(new_client);
        }
        
        Ok(())
    }

    async fn init_tables(&self) -> Result<()> {
        let client = self.get_client().await;
        client.batch_execute(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                username VARCHAR(255) UNIQUE NOT NULL,
                fullname VARCHAR(255) NOT NULL,
                following_count INTEGER NOT NULL DEFAULT 0,
                follower_count INTEGER NOT NULL DEFAULT 0,
                cool_comment TEXT[] DEFAULT '{}',
                cool_count INTEGER NOT NULL DEFAULT 0,
                bad_comment TEXT[] DEFAULT '{}',
                bad_count INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
            "#
        ).await?;
        Ok(())
    }

    pub async fn find_user(&self, username: &str) -> Result<Option<User>> {
        self.ensure_connection().await?;
        let client = self.get_client().await;
        
        let row = client
            .query_opt(
                "SELECT username, fullname, following_count, follower_count, 
                        cool_comment, cool_count, bad_comment, bad_count 
                 FROM users WHERE username = $1",
                &[&username],
            )
            .await?;
        
        if let Some(row) = row {
            let user = User {
                username: row.get("username"),
                fullname: row.get("fullname"),
                following_count: row.get("following_count"), // 캐스팅 제거
                follower_count: row.get("follower_count"),   // 캐스팅 제거
                cool_comment: row.get("cool_comment"),
                cool_count: row.get("cool_count"),           // 캐스팅 제거
                bad_comment: row.get("bad_comment"),
                bad_count: row.get("bad_count"),             // 캐스팅 제거
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    pub async fn create_user(&self, user: &User) -> Result<()> {
        self.ensure_connection().await?;
        let client = self.get_client().await;
        
        client
            .execute(
                "INSERT INTO users (username, fullname, following_count, follower_count, 
                                  cool_comment, cool_count, bad_comment, bad_count)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &user.username,
                    &user.fullname,
                    &user.following_count,    // 캐스팅 제거
                    &user.follower_count,     // 캐스팅 제거
                    &user.cool_comment,
                    &user.cool_count,         // 캐스팅 제거
                    &user.bad_comment,
                    &user.bad_count,          // 캐스팅 제거
                ],
            )
            .await?;
        Ok(())
    }
    
    // 문제2: negative_count 증가
    pub async fn increment_negative_count(&self, username: &str) -> Result<bool> {
        self.ensure_connection().await?;
        let client = self.get_client().await;
        
        let result = client
            .execute(
                "UPDATE users SET bad_count = bad_count + 1, updated_at = NOW() 
                 WHERE username = $1",
                &[&username],
            )
            .await?;
        
        Ok(result > 0)
    }
    
    // 문제2: cool_count 증가
    pub async fn increment_cool_count(&self, username: &str) -> Result<bool> {
        self.ensure_connection().await?;
        let client = self.get_client().await;
        
        let result = client
            .execute(
                "UPDATE users SET cool_count = cool_count + 1, updated_at = NOW() 
                 WHERE username = $1",
                &[&username],
            )
            .await?;
        
        Ok(result > 0)
    }
    
    // 문제3: negative_count 증가 + 댓글 추가
    pub async fn add_negative_feedback(&self, username: &str, comment: &str) -> Result<bool> {
        self.ensure_connection().await?;
        let client = self.get_client().await;
        
        let result = client
            .execute(
                "UPDATE users 
                 SET bad_count = bad_count + 1, 
                     bad_comment = array_append(bad_comment, $2),
                     updated_at = NOW() 
                 WHERE username = $1",
                &[&username, &comment],
            )
            .await?;
        
        Ok(result > 0)
    }
    
    // 문제4: cool_count 증가 + 댓글 추가
    pub async fn add_cool_feedback(&self, username: &str, comment: &str) -> Result<bool> {
        self.ensure_connection().await?;
        let client = self.get_client().await;
        
        let result = client
            .execute(
                "UPDATE users 
                 SET cool_count = cool_count + 1, 
                     cool_comment = array_append(cool_comment, $2),
                     updated_at = NOW() 
                 WHERE username = $1",
                &[&username, &comment],
            )
            .await?;
        
        Ok(result > 0)
    }
}