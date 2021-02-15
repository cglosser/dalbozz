use serenity::model::interactions::Interaction;

#[derive(Serialize)]
struct CommandResponse {
    #[serde(rename = "type")]
    type_id: u8,
    data: Option<DataContainer>,
}

#[derive(Serialize)]
struct DataContainer {
    content: String,
}

pub fn generate_response(
    show_user_command: bool,
    message: Option<&str>,
) -> String {
    let data = message.map(|t| DataContainer {
        content: t.to_string(),
    });

    let reply = match (show_user_command, message) {
        (false, Some(_)) => CommandResponse { type_id: 3, data },
        (false, None) => CommandResponse { type_id: 2, data },
        (true, Some(_)) => CommandResponse { type_id: 4, data },
        (true, None) => CommandResponse { type_id: 5, data },
    };

    serde_json::to_string(&reply).unwrap()
}

pub async fn send_response(
    command: &Interaction,
    show_user_command: bool,
    message: Option<&str>,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://discord.com/api/v8/interactions/{}/{}/callback",
        command.id, command.token
    );

    let post_data = generate_response(show_user_command, message);

    client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(post_data)
        .send()
        .await
}

// Configure commands.  Global commands are cached on an hourly basis
// on Discord's side, so any changes may take up to an hour to appear.
// Server-specific commands appear immediately.
pub async fn configure_commands(
    token: &str,
    app_id: &str,
    server_id: &Option<String>,
) {
    let client = reqwest::Client::new();
    let url = match server_id {
        None => format!(
            "https://discord.com/api/v8/applications/{}/commands",
            app_id
        ),
        Some(s) => format!(
            "https://discord.com/api/v8/applications/{}/guilds/{}/commands",
            app_id, s
        ),
    };
    let post_data = std::fs::read_to_string("init_commands.json").unwrap();
    let response = client
        .post(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bot {}", token))
        .header(reqwest::header::CONTENT_LENGTH, post_data.len())
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(post_data)
        .send()
        .await;

    match response {
        Ok(res) => {
            println!(
                "HTTP {}, text = {}",
                res.status(),
                res.text().await.unwrap_or("".to_string())
            );
        }
        Err(res) => {
            println!("HTTP Err = {:?}", res);
        }
    }
}
