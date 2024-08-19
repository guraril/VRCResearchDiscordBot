use chrono::{FixedOffset, Timelike, Utc};
use poise::serenity_prelude::{
    self as serenity,
    all::{ChannelId, Context, CreateMessage, EventHandler, GatewayIntents, Ready},
    async_trait,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
};

struct Data {}

#[derive(Debug, Deserialize, Serialize)]
struct Tokens {
    discord_token: String,
    channels: Vec<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReleaseCache {
    releases: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseUrl {
    html_url: String,
}

#[poise::command(slash_command, prefix_command)]
async fn research_bot(
    ctx: poise::Context<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
    command: String,
    sub_command: String,
    argument: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut tokens = Tokens {
        discord_token: "".to_string(),
        channels: Vec::with_capacity(8),
    };
    if let Ok(content) = fs::read_to_string("./tokens.json") {
        if let Ok(json) = serde_json::from_str(content.as_str()) {
            tokens = json;
        } else {
            print!("Please type discord bot token here> ");
            std::io::stdin().read_line(&mut tokens.discord_token).ok();
        }
    }

    match &*command {
        "repo_watcher" => match &*sub_command {
            "add" => match &*argument {
                "AvatarOptimizer" => tokens.channels[0] = ctx.channel_id().into(),
                "ModularAvatar" => tokens.channels[1] = ctx.channel_id().into(),
                "lilToon" => tokens.channels[2] = ctx.channel_id().into(),
                "TexTransTool" => tokens.channels[3] = ctx.channel_id().into(),
                "VRCSDK" => tokens.channels[4] = ctx.channel_id().into(),
                "VRCFury" => tokens.channels[5] = ctx.channel_id().into(),
                "lilycalInventory" => tokens.channels[6] = ctx.channel_id().into(),
                "FaceEmo" => tokens.channels[7] = ctx.channel_id().into(),
                _ => {
                    println!("unknown repo");
                }
            },
            _ => {
                println!("unknown subcommand")
            }
        },
        _ => {
            println!("unknown command")
        }
    }
    let mut file = File::create("./tokens.json").unwrap();
    file.write_all(serde_json::to_string(&tokens).unwrap().as_bytes())
        .unwrap();
    file.flush().unwrap();
    let response = format!("Notification enabled, at channel ID: {}", &ctx.channel_id());
    ctx.say(response).await?;
    Ok(())
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        tokio::spawn(async move {
            let jst = FixedOffset::east_opt(9 * 3600).unwrap();
            loop {
                let now = Utc::now().with_timezone(&jst);
                if now.time().hour() % 2 == 0 && now.time().minute() < 5 {
                    let mut cache = ReleaseCache {
                        releases: Vec::new(),
                    };
                    if let Ok(content) = fs::read_to_string("./cache.json") {
                        if let Ok(json) = serde_json::from_str(content.as_str()) {
                            cache = json;
                        };
                    }
                    let mut tokens = Tokens {
                        discord_token: "".to_string(),
                        channels: Vec::with_capacity(8),
                    };
                    if let Ok(content) = fs::read_to_string("./tokens.json") {
                        if let Ok(json) = serde_json::from_str(content.as_str()) {
                            tokens = json;
                        };
                    }
                    let mut new_cache = ReleaseCache {
                        releases: Vec::with_capacity(8),
                    };

                    let client = reqwest::Client::new();
                    let request_urls = [
                        "https://api.github.com/repos/anatawa12/AvatarOptimizer/releases/latest",
                        "https://api.github.com/repos/bdunderscore/modular-avatar/releases/latest",
                        "https://api.github.com/repos/lilxyzw/liltoon/releases/latest",
                        "https://api.github.com/repos/ReinaS-64892/TexTransTool/releases/latest",
                        "https://api.github.com/repos/vrchat/packages/releases/latest",
                        "https://api.github.com/repos/VRCFury/VRCFury/releases/latest",
                        "https://api.github.com/repos/lilxyzw/lilycalInventory/releases/latest",
                        "https://api.github.com/repos/suzuryg/face-emo/releases/latest",
                    ];
                    for (i, val) in request_urls.iter().enumerate() {
                        let response = client
                            .get(*val)
                            .header("User-Agent", "Awesome")
                            .send()
                            .await
                            .unwrap();
                        let body: ReleaseUrl =
                            serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
                        new_cache.releases.push(body.html_url);
                        if cache.releases.len() <= i {
                            cache.releases.push("".to_string());
                        }
                        if new_cache.releases[i] != cache.releases[i] {
                            println!("New Release found!: {}", &new_cache.releases[i]);
                            ChannelId::new(tokens.channels[i])
                                .send_message(
                                    &ctx.http,
                                    CreateMessage::new().content(&new_cache.releases[i]),
                                )
                                .await
                                .unwrap();
                        } else {
                            println!(
                                "No Updates found. latest release is: {}",
                                &new_cache.releases[i]
                            )
                        }
                    }
                    let mut file = File::create("./cache.json").unwrap();
                    file.write_all(serde_json::to_string(&new_cache).unwrap().as_bytes())
                        .unwrap();
                    file.flush().unwrap();
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            }
        });
    }
}

#[tokio::main]
async fn main() {
    let mut tokens = Tokens {
        discord_token: "".to_string(),
        channels: Vec::with_capacity(8),
    };
    if let Ok(content) = fs::read_to_string("./tokens.json") {
        if let Ok(json) = serde_json::from_str(content.as_str()) {
            tokens = json;
        }
    }
    if tokens.discord_token == *"" {
        println!("Please type discord bot token here> ");
        std::io::stdin().read_line(&mut tokens.discord_token).ok();
        tokens.discord_token = tokens.discord_token.replace("\n", "");

        if let Ok(mut file) = File::create("./tokens.json") {
            if let Ok(json) = serde_json::to_string(&tokens) {
                file.write_all(json.as_bytes()).ok();
            }
            file.flush().ok();
        };
    }
    let bot_token: String = tokens.discord_token;
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![research_bot()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();
    let mut client = serenity::ClientBuilder::new(&bot_token, GatewayIntents::non_privileged())
        .event_handler(Handler)
        .framework(framework)
        .await
        .unwrap();
    client.start().await.unwrap();
}
