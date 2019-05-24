use serde_json::Value;
use std::error::Error;
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use reqwest::{StatusCode, Client};
use std::fmt::Write;

pub struct AccountInfo {
    access_token: String,
    client_token: String,
    uuid: String,
}

pub fn auth(client: &Client, user: &str, password: &str) -> Result<AccountInfo, reqwest::Error> {
    let body: Value = json!({
        "agent": {
            "name": "Minecraft",
            "version": 1
        },
        "username": user,
        "password": password,
    });

    let mut resp = client.post("https://authserver.mojang.com/authenticate")
        .json(&body)
        .send()?;

    if resp.status() != StatusCode::OK {
        let d: Value = resp.json()?;
        println!("{:?}", d.to_string());
    }

    let body: Value = resp.json()?;

    let access_token = body.get("accessToken").unwrap().as_str().unwrap().to_owned();
    let client_token = body.get("clientToken").unwrap().as_str().unwrap().to_owned();
    let uuid = body.get("selectedProfile").unwrap().get("id").unwrap().as_str().unwrap().to_owned();

    Ok(AccountInfo { access_token, client_token, uuid })
}

pub fn join_server(client: &Client, info: AccountInfo, server_id: String, secret: &Vec<u8>, key: &Vec<u8>) -> Result<(), String> {
    let sha = create_mc_sha1(&[&"".to_ascii_lowercase().as_bytes(), &secret, &key]);

    let body: Value = json!({
        "accessToken": info.access_token,
        "selectedProfile": info.uuid,
        "serverId": sha
    });

    match client.post("https://sessionserver.mojang.com/session/minecraft/join").json(&body).send() {
        Ok(resp) => {
            if resp.status() != StatusCode::NO_CONTENT {
                return Err("Failed".to_owned())
            }
            return Ok(())
        },
        Err(e) => {
            return Err(e.description().to_owned());
        }
    }
    Ok(())
}

fn create_mc_sha1(input: &[&[u8]]) -> String {
    let mut hasher = Sha1::new();
    let mut data = [0u8; 20];

    for d in input.iter() {
        hasher.input(d);
    }

    hasher.result(&mut data);

    let mut negative = (data[0] & 0x80) == 0x80;

    if negative {
        twos_complement(&mut data);
    }

    let mut hex = hex::encode(data);

    if negative {
        hex.insert_str(0, "-")
    }

    hex
}

fn twos_complement(p: &mut [u8]) {
    let mut carry = true;
    for i in p.iter_mut().rev() {
        *i = !*i;
        if carry {
            carry = *i == 0xFFu8;
            *i += 1
        }
    }
}