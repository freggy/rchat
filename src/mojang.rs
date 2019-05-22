use serde_json::Value;
use std::error::Error;

extern crate reqwest;

pub struct TokenContainer {
    access_token: String,
    client_token: String
}

pub fn auth(user: &str, password: &str) -> Result<TokenContainer, reqwest::Error> {
    let client = reqwest::Client::new();

    let body: Value = json!({
        "agent": {
            "name": "Minecraft",
            "version": 1
        },
        "username": user,
        "password": password,
    });

    let resp: Value = client.post("https://authserver.mojang.com/authenticate")
        .json(&body)
        .send()?
        .json()?;

    let access_token = resp.get("accessToken").unwrap().as_str().unwrap().to_owned();
    let client_token = resp.get("clientToken").unwrap().as_str().unwrap().to_owned();

    Ok(TokenContainer { access_token, client_token })
}