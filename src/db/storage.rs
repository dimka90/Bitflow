use sqlx::PgPool;
use anyhow::Result;
use crate::util::hash::keccak_256;
use crate::dim::manifest::{self, DimManifest};
use crate::db::storage::manifest::ChunkInfo;
pub struct PgStorage{
pub pool: PgPool,
}

impl  PgStorage {
    pub async fn   new(database_url: &str)->Result<Self>{
        let pool = PgPool::connect(database_url).await?;
        Ok(Self{
            pool
        })
    }

     pub async fn store_manifest(&self, manifest: &DimManifest) -> Result<()> {
        let manifest_bytes = rmp_serde::to_vec(manifest)?;
        let manifest_hash = keccak_256(&manifest_bytes);

        let file_id: i32 = sqlx::query_scalar!(
            r#"
            INSERT INTO files (file_name, file_size, chunk_size, manifest_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            manifest.file_name,
            manifest.file_size as i64,
            manifest.chunk_size as i32,
            &manifest_hash[..],
        )
        .fetch_one(&self.pool)
        .await?;

        for chunk in &manifest.chunks {
            self.store_chunk(file_id, chunk).await?;
        }

        Ok(())
    }

    async fn store_chunk(&self, file_id: i32, chunk: &ChunkInfo) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO chunks (file_id, chunk_index, chunk_hash, chunk_size)
            VALUES ($1, $2, $3, $4)
            "#,
            file_id,
            chunk.index as i32,
            &chunk.hash[..],
            chunk.size as i32,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

    
