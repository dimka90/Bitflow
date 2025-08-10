use sqlx::PgPool;
use anyhow::Result;
use crate::util::hash::keccak_256;
use crate::dim::manifest::{ DimManifest, ChunkInfo};
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

           if let Some(existing_file_id) = sqlx::query_scalar!(
        "SELECT id FROM files WHERE manifest_hash = $1",
        &manifest_hash[..]
          )
        .fetch_optional(&self.pool)
                .await?
                 {
            println!("Content Id already exist {:?}", existing_file_id);
            return Ok(());
                }
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
    
    pub async fn load_manifest_by_hash(&self, manifest_hash: &[u8]) -> Result<Option<DimManifest>> {
        let rec = sqlx::query!(
            r#"
            SELECT id, file_name, file_size, chunk_size
            FROM files
            WHERE manifest_hash = $1
            "#,
            manifest_hash
        )
        .fetch_optional(&self.pool)
        .await?;

        let rec = match rec {
            Some(r) => r,
            None => return Ok(None),
        };

        let file_id = rec.id;
        let file_name = rec.file_name;
        let file_size = rec.file_size as u64;
        let chunk_size = rec.chunk_size as usize;

        let rows = sqlx::query!(
            r#"
            SELECT chunk_index, chunk_hash, chunk_size
            FROM chunks
            WHERE file_id = $1
            ORDER BY chunk_index
            "#,
            file_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut chunks: Vec<ChunkInfo> = Vec::with_capacity(rows.len());
        for row in rows {
            chunks.push(ChunkInfo {
                index: row.chunk_index as usize,
                hash: row.chunk_hash,
                size: row.chunk_size as usize,
            });
        }

        let manifest = DimManifest {
            file_name,
            file_size,
            chunk_size,
            chunks,
        };

        Ok(Some(manifest))
    }

    pub async fn load_manifest_by_id(&self, file_id: i32) -> Result<DimManifest> {
        let rec = sqlx::query!(
            r#"
            SELECT file_name, file_size, chunk_size
            FROM files
            WHERE id = $1
            "#,
            file_id
        )
        .fetch_one(&self.pool)
        .await?;

        let file_name = rec.file_name;
        let file_size = rec.file_size as u64;
        let chunk_size = rec.chunk_size as usize;

        let rows = sqlx::query!(
            r#"
            SELECT chunk_index, chunk_hash, chunk_size
            FROM chunks
            WHERE file_id = $1
            ORDER BY chunk_index
            "#,
            file_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut chunks: Vec<ChunkInfo> = Vec::with_capacity(rows.len());
        for row in rows {
            chunks.push(ChunkInfo {
                index: row.chunk_index as usize,
                hash: row.chunk_hash,
                size: row.chunk_size as usize,
            });
        }

        Ok(DimManifest {
            file_name,
            file_size,
            chunk_size,
            chunks,
        })
    }
}

    
