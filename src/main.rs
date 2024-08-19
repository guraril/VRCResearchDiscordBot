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
struct GitHubRequest {
    name: String,
    url: String,
}
#[derive(Debug, Deserialize, Serialize)]
struct RequestLists {
    github_requests: Vec<GitHubRequest>,
}

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

fn load_request_lists() -> RequestLists {
    let mut lists = RequestLists {
        github_requests: Vec::new(),
    };
    if let Ok(content) = fs::read_to_string("./request_lists.json") {
        if let Ok(json) = serde_json::from_str(content.as_str()) {
            lists = json;
        }
    }
    lists
}

fn save_cache(new_cache: &ReleaseCache) {
    if let Ok(mut file) = File::create("./cache.json") {
        if let Ok(json) = serde_json::to_string(&new_cache) {
            file.write_all(json.as_bytes()).ok();
        }
        file.flush().ok();
    }
}

fn load_cache() -> ReleaseCache {
    let mut cache = ReleaseCache {
        releases: Vec::new(),
    };
    if let Ok(content) = fs::read_to_string("./cache.json") {
        if let Ok(json) = serde_json::from_str(content.as_str()) {
            cache = json;
        };
    };
    cache
}

fn save_tokens(tokens: &Tokens) {
    if let Ok(mut file) = File::create("./tokens.json") {
        if let Ok(json) = serde_json::to_string(&tokens) {
            file.write_all(json.as_bytes()).ok();
        }
        file.flush().ok();
    };
}

fn load_tokens() -> Tokens {
    let mut tokens = Tokens {
        discord_token: "".to_string(),
        channels: Vec::with_capacity(8),
    };
    if let Ok(content) = fs::read_to_string("./tokens.json") {
        if let Ok(data) = serde_json::from_str(content.as_str()) {
            tokens = data;
        };
    }
    tokens
}

#[poise::command(slash_command, prefix_command)]
async fn research_bot(
    ctx: poise::Context<'_, Data, Box<dyn std::error::Error + Send + Sync>>,
    command: String,
    sub_command: String,
    argument: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut tokens = load_tokens();
    let request_lists = load_request_lists();

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
                println!("unknown subcommand");
            }
        },
        _ => {
            println!("unknown command");
        }
    }
    save_tokens(&tokens);
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
                    let mut cache = load_cache();
                    let tokens = load_tokens();
                    let request_lists = load_request_lists();
                    let mut new_cache = ReleaseCache {
                        releases: Vec::with_capacity(request_lists.github_requests.len()),
                    };

                    let client = reqwest::Client::new();
                    for (i, val) in request_lists.github_requests.iter().enumerate() {
                        println!("{}: {}", &val.name, &val.url);
                        if let Ok(response) = client
                            .get(&val.url)
                            .header("User-Agent", "Awesome")
                            .send()
                            .await
                        {
                            if let Ok(json) =
                                serde_json::from_str(response.text().await.unwrap().as_str())
                            {
                                let body: ReleaseUrl = json;
                                new_cache.releases.push(body.html_url);
                            }
                        }
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
                    save_cache(&new_cache);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            }
        });
    }
}

#[tokio::main]
async fn main() {
    let mut tokens = load_tokens();
    if tokens.discord_token == *"" {
        println!("Please type discord bot token here> ");
        std::io::stdin().read_line(&mut tokens.discord_token).ok();
        tokens.discord_token = tokens.discord_token.replace("\n", "");

        save_tokens(&tokens);
    }
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
    if let Ok(mut client) =
        serenity::ClientBuilder::new(&tokens.discord_token, GatewayIntents::non_privileged())
            .event_handler(Handler)
            .framework(framework)
            .await
    {
        if (client.start().await).is_ok() {}
    }
}
