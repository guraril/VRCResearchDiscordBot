// TODO: 各変数が目的にあった名前か確認する
use chrono::{Timelike, Utc};
use log::{error, info, warn};
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
struct ReleaseInfo {
    name: String,
    url: String,
}
// 厳密にはこれらは違うものですが、構造が全く同じなので共通化しました
type GitHubRequest = ReleaseInfo;

#[derive(Debug, Deserialize, Serialize)]
struct RequestLists {
    github_requests: Vec<GitHubRequest>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Tokens {
    discord_token: String,
    channels: Vec<u64>,
    github_release_notifications: Vec<GitHubReleaseNotifications>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GitHubReleaseNotifications {
    name: String,
    channel_id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReleaseCache {
    releases: Vec<ReleaseInfo>,
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
    }
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
        github_release_notifications: Vec::new(),
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
    // let request_lists = load_request_lists();

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
                    warn!("unknown repo");
                }
            },
            _ => {
                warn!("unknown subcommand");
            }
        },
        _ => {
            warn!("unknown command");
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
            loop {
                let now = Utc::now();
                if now.time().hour() % 2 == 0 && now.time().minute() < 5 {
                    let cache = load_cache();
                    let request_lists = load_request_lists();
                    let tokens = load_tokens();
                    let mut new_cache: ReleaseCache = ReleaseCache {
                        releases: Vec::new(),
                    };
                    let client = reqwest::Client::new();
                    for (i, val) in request_lists.github_requests.iter().enumerate() {
                        new_cache.releases.push(ReleaseInfo {
                            name: String::from(&val.name),
                            url: String::from(""),
                        });
                        if let Ok(response) = client
                            .get(&val.url)
                            .header("User-Agent", "Awesome")
                            .send()
                            .await
                        {
                            if let Ok(text) = response.text().await {
                                if let Ok(json) = serde_json::from_str(&text) {
                                    let body: ReleaseUrl = json;
                                    new_cache.releases[i].url = String::from(&body.html_url);
                                    info!("Fetched url is {}", &body.html_url);
                                } else {
                                    error!("Failed to get json from response");
                                    continue;
                                }
                            } else {
                                error!("Failed to get text from response");
                                continue;
                            }
                        } else {
                            error!("Failed to get response from GitHub");
                            continue;
                        };
                        if let Some(cache_release) =
                            cache.releases.iter().find(|&x| x.name == val.name)
                        {
                            if cache_release.url != new_cache.releases[i].url {
                                info!("New release found! repo: {}", &new_cache.releases[i].name);
                                if let Some(github_release_notification) = tokens
                                    .github_release_notifications
                                    .iter()
                                    .find(|&x| x.name == val.name)
                                {
                                    if let Ok(_msg) =
                                        ChannelId::new(github_release_notification.channel_id)
                                            .send_message(
                                                &ctx.http,
                                                CreateMessage::new().content(
                                                    String::from("New release found!\n{}")
                                                        + &new_cache.releases[i].url,
                                                ),
                                            )
                                            .await
                                    {
                                        info!(
                                            "Message was successfully sent. channel_id: {}",
                                            github_release_notification.channel_id
                                        );
                                    } else {
                                        error!("Failed to send message to discord channel. channel_id: {}", github_release_notification.channel_id);
                                    }
                                } else {
                                    warn!("Notification is not specified for this repository.");
                                }
                            } else {
                                info!("No updates found. repo: {}", new_cache.releases[i].name);
                            }
                        } else {
                            info!("New release found! repo: {}", new_cache.releases[i].name);
                            if let Some(github_release_notification) = tokens
                                .github_release_notifications
                                .iter()
                                .find(|&x| x.name == val.name)
                            {
                                if let Ok(_msg) =
                                    ChannelId::new(github_release_notification.channel_id)
                                        .send_message(
                                            &ctx.http,
                                            CreateMessage::new().content(format!(
                                                "New release found!\n {}",
                                                &new_cache.releases[i].url
                                            )),
                                        )
                                        .await
                                {
                                    info!(
                                        "Message was successfully sent. channel_id: {}",
                                        github_release_notification.channel_id
                                    );
                                } else {
                                    error!(
                                        "Failed to send message to discord channel. channel_id: {}",
                                        github_release_notification.channel_id
                                    );
                                }
                            } else {
                                warn!("Notification is not specified for this repository.");
                            }
                        };
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
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let ts = buf.timestamp();
            writeln!(buf, "[{} {}]: {}", ts, record.level(), record.args())
        })
        .init();
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
