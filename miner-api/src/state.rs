// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
use sqlx::SqlitePool;
use std::sync::Arc;
use ed25519_dalek::{SigningKey, VerifyingKey};
use std::sync::Arc as StdArc;

use crate::models::{MinerInfo, Claims};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub jwt_secret: Vec<u8>,
    pub signing_key: StdArc<ed25519_dalek::SigningKey>,
    pub verifying_key: StdArc<ed25519_dalek::VerifyingKey>,
}

impl AppState {
    pub async fn new(pool: sqlx::SqlitePool, jwt_secret: Vec<u8>) -> anyhow::Result<Self> {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let signing_key = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            pool: sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:/srv/beyin/beyin.db".to_string())).await?,
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "nemes-jwt-secret-change-in-production".to_string()).into_bytes(),
            signing_key: std::sync::Arc::new(ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng)),
            verifying_key: std::sync::Arc::new(ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng).verifying_key()),
        }
    }

    pub async fn get_miner(&self, miner_id: &str) -> anyhow::Result<Option<crate::models::MinerInfo>> {
        let miner = sqlx::query_as!(
            crate::models::MinerInfo,
            r#"SELECT id, cuzdan, makine_id, pay, api_hakki, bakiye, last_heartbeat, created_at as olusturma_zamani 
               FROM miner WHERE id = ?"#,
            miner_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(miner)
    }

    pub async fn create_miner(&self, cuzdan: &str, makine_id: &str, eposta: Option<String>) -> anyhow::Result<(String, String)> {
        let miner_id = format!("miner-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
        let token = crate::auth::create_token(&miner_id, "")?;
        
        sqlx::query!(
            r#"INSERT INTO miner (id, cuzdan, makine_id, eposta, token, pay, api_hakki, bakiye, created_at, last_heartbeat) 
               VALUES (?, ?, ?, ?, ?, 0, 0, 0.0, ?, ?)"#,
            miner_id, cuzdan, miner_id, chrono::Utc::now().timestamp(), chrono::Utc::now().timestamp()
        )
        .execute(&self.pool)
        .await?;

        Ok((miner_id.to_string(), self.create_token(&miner_id).await?))
    }

    pub async fn update_heartbeat(&self, miner_id: &str) -> anyhow::Result<()> {
        sqlx::query!("UPDATE miner SET last_heartbeat = ? WHERE id = ?", chrono::Utc::now().timestamp(), miner_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_gorev(&self) -> anyhow::Result<Option<crate::models::GorevResponse>> {
        // TODO: Implement from beyin_sirasi.py logic
        Ok(None)
    }

    pub async fn submit_kanit(&self, miner_id: &str, kanit: crate::models::KanitRequest) -> anyhow::Result<u64> {
        // Verify kanit here
        // For now just increment pay
        sqlx::query!("UPDATE miner SET pay = pay + 1, api_hakki = api_hakki + 1 WHERE id = ?", miner_id)
            .execute(&self.pool)
            .await?;
        
        let row = sqlx::query!("SELECT pay FROM miner WHERE id = ?", miner_id)
            .fetch_one(&self.pool)
            .await?;
        
        Ok(row.pay as u64)
    }

    pub async fn get_status(&self, miner_id: &str) -> anyhow::Result<crate::models::MinerStatusResponse> {
        let miner = self.get_miner(miner_id).await?.ok_or_else(|| anyhow::anyhow!("Miner not found"))?;
        let bagli = (chrono::Utc::now().timestamp() - miner.last_heartbeat as i64) < 90;
        
        Ok(crate::models::MinerStatusResponse {
            miner_id: miner.id,
            pay: miner.pay as u64,
            api_hakki: miner.api_hakki as u64,
            bakiye: miner.bakiye,
            bagli: bagli,
            son_heartbeat: miner.last_heartbeat as u64,
            miner_id: miner.id,
        })
    }
}