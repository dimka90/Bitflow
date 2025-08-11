use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use crate::dim::manifest::DimManifest;
use crate::p2p::network::{Client, EventLoop, Event};
use libp2p::Multiaddr;
use std::fs;
use futures::StreamExt;

pub struct P2PFileManager {
    network_client: Client,
    event_loop: EventLoop,
    local_files: Arc<RwLock<HashMap<String, DimManifest>>>,
    shared_files: Arc<RwLock<HashMap<String, DimManifest>>>,
    event_stream: futures::stream::BoxStream<'static, Event>,
}

impl P2PFileManager {
    pub async fn new(secret_key_seed: Option<u8>) -> Result<Self> {
        let (client, event_stream, event_loop) = crate::p2p::network::new(secret_key_seed).await?;
        
        Ok(Self {
            network_client: client,
            event_loop,
            local_files: Arc::new(RwLock::new(HashMap::new())),
            shared_files: Arc::new(RwLock::new(HashMap::new())),
            event_stream: event_stream.boxed(),
        })
    }

    /// Start processing network events in the background
    pub async fn start_event_processing(&mut self) -> Result<()> {
        let mut event_stream = std::mem::replace(&mut self.event_stream, futures::stream::pending().boxed());
        
        tokio::spawn(async move {
            println!("🔄 Starting network event processing...");
            while let Some(event) = event_stream.next().await {
                match event {
                    Event::InboundRequest { request, channel: _ } => {
                        println!("📥 Received file request for: {}", request);
                        // For now, just log the request
                        // In a real implementation, you'd look up the file and send it
                        println!("⚠️  File request received but not yet implemented");
                    }
                }
            }
            println!("🔄 Network event processing stopped");
        });
        
        Ok(())
    }

    /// Start listening on the given address
    pub async fn start_listening(&mut self, addr: Multiaddr) -> Result<()> {
        self.network_client.start_listening(addr).await?;
        println!("🌐 P2P network started and listening");
        Ok(())
    }

    /// Share a file on the P2P network
    pub async fn share_file(&mut self, file_path: &str) -> Result<()> {
        println!("🔍 Checking if file exists: {}", file_path);
        // Check if file exists
        if !Path::new(file_path).exists() {
            return Err(anyhow::anyhow!("File not found: {}", file_path));
        }
        println!("✅ File exists, proceeding with manifest creation...");

        // Create manifest if it doesn't exist
        let manifest = if let Some((file_name, _)) = file_path.split_once(".") {
            let dim_file = format!("{}.dim", file_name);
            println!("📝 Looking for existing manifest: {}", dim_file);
            if Path::new(&dim_file).exists() {
                // Load existing manifest
                println!("📖 Loading existing manifest...");
                let manifest = DimManifest::load_dim_manifest(&dim_file)?;
                println!("✅ Manifest loaded successfully");
                manifest
            } else {
                // Create new manifest
                println!("🆕 Creating new manifest...");
                let manifest = DimManifest::create_manifest(file_path)?;
                println!("💾 Saving manifest to: {}", dim_file);
                manifest.save_to_dim_manifest(&dim_file)?;
                println!("✅ Manifest created and saved successfully");
                manifest
            }
        } else {
            return Err(anyhow::anyhow!("Invalid file path: {}", file_path));
        };

        println!("💾 Storing file in local files registry...");
        // Store in local files
        {
            let mut local_files = self.local_files.write().await;
            local_files.insert(file_path.to_string(), manifest.clone());
        }
        println!("✅ File stored in local registry");

        // Start providing the file in the DHT
        let file_name = manifest.file_name.clone();
        println!("🌐 Starting to provide file '{}' in DHT...", file_name);
        self.network_client.start_providing(file_name.clone()).await;
        println!("✅ File provision started in DHT");
        
        println!("💾 Storing file in shared files registry...");
        // Store in shared files
        {
            let mut shared_files = self.shared_files.write().await;
            shared_files.insert(file_name.clone(), manifest);
        }
        println!("✅ File stored in shared registry");

        println!("✅ File '{}' is now being shared on the P2P network!", file_name);
        Ok(())
    }

    /// Discover and download a file from the P2P network
    pub async fn discover_and_download(&mut self, file_name: &str) -> Result<()> {
        println!("🔍 Searching for file '{}' on the P2P network...", file_name);
        
        // Search for providers in the DHT
        let providers = self.network_client.get_providers(file_name.to_string()).await;
        
        if providers.is_empty() {
            println!("❌ No providers found for file '{}'", file_name);
            return Err(anyhow::anyhow!("File not found on network"));
        }

        println!("✅ Found {} provider(s) for file '{}'", providers.len(), file_name);
        
        // Try to download from the first available provider
        let peer_id = providers.iter().next().unwrap();
        println!("📥 Downloading from peer: {}", peer_id);
        
        match self.network_client.request_file(*peer_id, file_name.to_string()).await {
            Ok(file_content) => {
                // Save the downloaded file
                let file_path = format!("downloaded_{}", file_name);
                fs::write(&file_path, &file_content)?;
                
                // Create manifest for the downloaded file
                let manifest = DimManifest::create_manifest(&file_path)?;
                let dim_file = format!("downloaded_{}.dim", file_name);
                manifest.save_to_dim_manifest(&dim_file)?;
                
                // Store in local files
                {
                    let mut local_files = self.local_files.write().await;
                    local_files.insert(file_path.clone(), manifest.clone());
                }
                
                // Start sharing the downloaded file automatically
                self.network_client.start_providing(file_name.to_string()).await;
                
                // Store in shared files
                {
                    let mut shared_files = self.shared_files.write().await;
                    shared_files.insert(file_name.to_string(), manifest);
                }
                
                println!("✅ File '{}' downloaded and now being shared!", file_name);
                Ok(())
            }
            Err(e) => {
                println!("❌ Failed to download file: {}", e);
                Err(anyhow::anyhow!("Download failed: {}", e))
            }
        }
    }

    /// List all available files on the network
    pub async fn list_available_files(&self) -> Result<Vec<String>> {
        // For now, we'll return files we know about
        // In a real implementation, you'd query the DHT for all available files
        let shared_files = self.shared_files.read().await;
        let local_files = self.local_files.read().await;
        
        let mut all_files = Vec::new();
        
        for file_name in shared_files.keys() {
            all_files.push(file_name.clone());
        }
        
        for file_name in local_files.keys() {
            if !all_files.contains(file_name) {
                all_files.push(file_name.clone());
            }
        }
        
        Ok(all_files)
    }

    /// Get the event loop for processing network events
    pub fn get_event_loop(self) -> EventLoop {
        self.event_loop
    }

    /// Get the network client for making requests
    pub fn get_network_client(&self) -> &Client {
        &self.network_client
    }
}
