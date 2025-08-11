mod util;
mod dim;
mod db;
mod p2p;

use std::env;
use std::path::Path;
use dotenv::dotenv;
use anyhow::Result;
use libp2p::Multiaddr;
use crate::p2p::file_manager::P2PFileManager;


#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Bitflow starting up...");
    dotenv().ok();
    println!("✅ Environment loaded");
    
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    println!("📝 Command line arguments: {:?}", args);
    
    if args.len() < 2 {
        // No file specified - just join the network for discovery
        println!("🚀 Starting Bitflow P2P node (discovery mode)...");
        run_discovery_mode().await?;
    } else {
        // File specified - share or download the file
        let file_path = &args[1];
        println!("🚀 Starting Bitflow P2P node with file: {}", file_path);
        println!("🔧 About to call run_file_mode...");
        run_file_mode(file_path).await?;
        println!("✅ run_file_mode completed");
    }
    
    println!("🏁 Main function completed");
    Ok(())
}

async fn run_discovery_mode() -> Result<()> {
    // Create P2P file manager
    let mut file_manager = P2PFileManager::new(None).await?;
    
    // Start listening on default address
    println!("🔧 Starting to listen on address...");
    let addr: Multiaddr = "/ip4/127.0.0.1/tcp/8080".parse()?;
    println!("🌐 Address parsed: {}", addr);
    println!("🔧 Calling start_listening...");
    file_manager.start_listening(addr).await?;
    println!("✅ start_listening completed");
    
    // Start processing network events
    println!("🔧 Starting network event processing...");
    file_manager.start_event_processing().await?;
    println!("✅ Network event processing started");
    
    println!("🌐 P2P node started in discovery mode");
    println!("📡 Listening for network activity...");
    println!("💡 To download a file, run: cargo run <filename>");
    
    // Start the event loop in a separate task
    let event_loop = file_manager.get_event_loop();
    tokio::spawn(async move {
        println!("🔄 Event loop starting...");
        event_loop.run().await;
        println!("🔄 Event loop stopped");
    });
    
    // Wait for a reasonable amount of time instead of infinite loop
    println!("⏰ Discovery mode active for 60 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    println!("✅ Discovery mode completed");
    
    Ok(())
}

async fn run_file_mode(file_path: &str) -> Result<()> {
    println!("🔧 run_file_mode started with file: {}", file_path);
    // Create P2P file manager
    println!("🔧 Creating P2P file manager...");
    let mut file_manager = P2PFileManager::new(None).await?;
    println!("✅ P2P file manager created successfully");
    
    // Start listening on default address
    println!("🔧 Starting to listen on address...");
    let addr: Multiaddr = "/ip4/127.0.0.1/tcp/8080".parse()?;
    println!("🌐 Address parsed: {}", addr);
    println!("🔧 Calling start_listening...");
    file_manager.start_listening(addr).await?;
    println!("✅ start_listening completed");
    
    // Start processing network events
    println!("🔧 Starting network event processing...");
    file_manager.start_event_processing().await?;
    println!("✅ Network event processing started");
    
    // Check if file exists locally
    if Path::new(file_path).exists() {
        // File exists locally - share it
        println!("📁 File exists locally - sharing on P2P network...");
        println!("🔧 Creating manifest and sharing file...");
        file_manager.share_file(file_path).await?;
        
        println!("✅ File is now being shared!");
        println!("🌐 Other peers can discover and download it");
    } else {
        // File doesn't exist locally - try to download it
        println!("📥 File not found locally - searching P2P network...");
        
        match file_manager.discover_and_download(file_path).await {
            Ok(_) => {
                println!("✅ File downloaded and now being shared!");
            }
            Err(e) => {
                println!("❌ Failed to download file: {}", e);
                println!("💡 Make sure another peer is sharing this file");
                return Err(e);
            }
        }
    }
    
    // Start the event loop in a separate task
    println!("🔧 Starting event loop...");
    let event_loop = file_manager.get_event_loop();
    tokio::spawn(async move {
        println!("🔄 Event loop starting...");
        event_loop.run().await;
        println!("🔄 Event loop stopped");
    });
    println!("✅ Event loop started in background");
    
    println!("⏹️  File mode active for 60 seconds...");
    
    // Wait for a reasonable amount of time instead of infinite loop
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    println!("✅ File mode completed");
    
    Ok(())
}


