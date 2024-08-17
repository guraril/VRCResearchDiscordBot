use chrono::{FixedOffset, Timelike, Utc};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{
    all::{ChannelId, Context, CreateMessage, EventHandler, GatewayIntents, Ready},
    async_trait,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
};
use tokio;

struct Data {}

#[derive(Debug, Deserialize)]
struct Tokens {
    discord_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReleaseCache {
    releases: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseUrl {
    html_url: String,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type CommandContext<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: CommandContext<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let channel_id: u64 = 1274288677845209153;
        tokio::spawn(async move {
            let jst = FixedOffset::east_opt(9 * 3600).unwrap();
            loop {
                let now = Utc::now().with_timezone(&jst);
                if now.time().hour() % 1 == 0 && now.time().minute() < 5 {
                    let mut cache: ReleaseCache;
                    match fs::read_to_string("./cache.json") {
                        Ok(content) => {
                            cache = serde_json::from_str(content.as_str()).unwrap();
                        }
                        Err(_) => {
                            cache = ReleaseCache {
                                releases: Vec::new(),
                            }
                        }
                    }
                    let mut new_cache = ReleaseCache {
                        releases: Vec::with_capacity(8),
                    };

                    let client = reqwest::Client::new();
                    let request_urls = vec![
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
                            ChannelId::new(channel_id)
                                .send_message(
                                    &ctx.http,
                                    CreateMessage::new().content(&new_cache.releases[i]),
                                )
                                .await
                                .unwrap();
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
    let tokens: Tokens =
        serde_json::from_str(fs::read_to_string("./tokens.json").unwrap().as_str()).unwrap();
    let bot_token: String = tokens.discord_token;
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age()],
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
