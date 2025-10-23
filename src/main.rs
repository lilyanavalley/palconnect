
use poise::serenity_prelude as serenity;
use reqwest::Client;
use serde::Deserialize;
use std::env;


// Data structure that will be accessible in all command invocations
pub struct BotData {
  http_client: Client,
  palworld_api_url: String,
  admin_password: String,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, BotData, Error>;

// PalWorld API response structures
#[derive(Debug, Deserialize)]
struct ServerInfo {
  version: String,
  servername: String,
  description: String,
}

#[derive(Debug, Deserialize)]
struct PlayersResponse {
  players: Vec<Player>,
}

#[derive(Debug, Deserialize)]
struct Player {
  name: String,
  #[serde(rename = "playerId")]
  player_id: String,
  #[serde(rename = "userId")]
  user_id: String,
  ip: String,
  ping: f64,
  location_x: f64,
  location_y: f64,
  level: u32,
}

/// Show current player count on the PalWorld server
#[poise::command(slash_command)]
async fn players(ctx: Context<'_>) -> Result<(), Error> {
  ctx.defer().await?;
  
  let data = ctx.data();
  let url = format!("{}/v1/api/players", data.palworld_api_url);
  
  match data.http_client
    .get(&url)
    .basic_auth("admin", Some(&data.admin_password))
    .send()
    .await {
    Ok(response) => {
      match response.json::<PlayersResponse>().await {
        Ok(players_data) => {
          let player_count = players_data.players.len();
          let player_list = if players_data.players.is_empty() {
            "No players currently online".to_string()
          } else {
            players_data.players.iter()
              .map(|p| format!("• {} (Level {})", p.name, p.level))
              .collect::<Vec<String>>()
              .join("\n")
          };
          
          let embed = serenity::CreateEmbed::new()
            .title("🎮 PalWorld Server Status")
            .field("Players Online", player_count.to_string(), true)
            .field("Player List", player_list, false)
            .color(if player_count > 0 { 0x00ff00 } else { 0xff0000 })
            .timestamp(serenity::Timestamp::now());
          
          ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
          ctx.send(poise::CreateReply::default()
            .content(format!("❌ Failed to parse server response: {}", e))
            .ephemeral(true)).await?;
        }
      }
    }
    Err(e) => {
      ctx.send(poise::CreateReply::default()
        .content(format!("❌ Failed to connect to PalWorld server: {}", e))
        .ephemeral(true)).await?;
    }
  }
  
  Ok(())
}

/// Show server information
#[poise::command(slash_command)]
async fn serverinfo(ctx: Context<'_>) -> Result<(), Error> {
  ctx.defer().await?;
  
  let data = ctx.data();
  let url = format!("{}/v1/api/info", data.palworld_api_url);
  
  match data.http_client
    .get(&url)
    .basic_auth("admin", Some(&data.admin_password))
    .send()
    .await {
    Ok(response) => {
      match response.json::<ServerInfo>().await {
        Ok(server_info) => {
          let embed = serenity::CreateEmbed::new()
            .title("🏰 Server Information")
            .field("Server Name", &server_info.servername, true)
            .field("Version", &server_info.version, true)
            .field("Description", &server_info.description, false)
            .color(0x0099ff)
            .timestamp(serenity::Timestamp::now());
          
          ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
          ctx.send(poise::CreateReply::default()
            .content(format!("❌ Failed to parse server response: {}", e))
            .ephemeral(true)).await?;
        }
      }
    }
    Err(e) => {
      ctx.send(poise::CreateReply::default()
        .content(format!("❌ Failed to connect to PalWorld server: {}", e))
        .ephemeral(true)).await?;
    }
  }
  
  Ok(())
}

/// Show help information
#[poise::command(slash_command)]
async fn help(ctx: Context<'_>) -> Result<(), Error> {
  let embed = serenity::CreateEmbed::new()
    .title("🤖 PalConnect Bot Help")
    .description("A Discord bot for monitoring your PalWorld dedicated server")
    .field("/players", "Show current online players and count", false)
    .field("/serverinfo", "Display server information", false)
    .field("/help", "Show this help message", false)
    .color(0x7289da)
    .footer(serenity::CreateEmbedFooter::new("PalConnect Bot v0.1.0"));
  
  ctx.send(poise::CreateReply::default().embed(embed)).await?;
  Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
  // Load environment variables from .env file
  dotenv::dotenv().ok();
  
  // Load environment variables
  let discord_token = env::var("DISCORD_TOKEN")
    .expect("Expected DISCORD_TOKEN environment variable");
  
  let palworld_api_url = env::var("PALWORLD_API_URL")
    .unwrap_or_else(|_| "http://localhost:8212".to_string());
  
  let admin_password = env::var("PALWORLD_ADMIN_PASSWORD")
    .expect("Expected PALWORLD_ADMIN_PASSWORD environment variable");
  
  println!("🚀 Starting PalConnect bot...");
  println!("📡 PalWorld API URL: {}", palworld_api_url);
  
  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: vec![players(), serverinfo(), help()],
      ..Default::default()
    })
    .setup(move |ctx, _ready, framework| {
      let palworld_api_url = palworld_api_url.clone();
      let admin_password = admin_password.clone();
      Box::pin(async move {
        poise::builtins::register_globally(ctx, &framework.options().commands).await?;
        Ok(BotData {
          http_client: Client::new(),
          palworld_api_url,
          admin_password,
        })
      })
    })
    .build();
  
  let intents = serenity::GatewayIntents::non_privileged();
  let client = serenity::ClientBuilder::new(discord_token, intents)
    .framework(framework)
    .await;
  
  match client {
    Ok(mut client) => {
      println!("✅ Bot connected successfully!");
      client.start().await?;
    }
    Err(e) => {
      eprintln!("❌ Failed to create client: {}", e);
    }
  }
  
  Ok(())
}
