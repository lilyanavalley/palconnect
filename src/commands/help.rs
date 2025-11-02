
use poise::serenity_prelude as serenity;

use crate::{Context, Error};


/// Show help information
#[poise::command(slash_command)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let embed = serenity::CreateEmbed::new()
        .title("🤖 PalConnect Bot Help")
        .description("A Discord bot for monitoring your PalWorld dedicated server")
        .field("/players", "Show current online players and count", false)
        .field("/serverinfo", "Display server information", false)
        .field("/start", "Start the PalWorld server", false)
        .field("/stop", "Stop the PalWorld server", false)
        .field("/forcestop", "Force stop the PalWorld server", false)
        .field("/help", "Show this help message", false)
        .color(0x7289da)
        .footer(serenity::CreateEmbedFooter::new(concat!(
            "PalConnect Bot ",
            env!("CARGO_PKG_VERSION")
        )));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
