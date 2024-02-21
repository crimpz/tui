use reqwest::cookie::Jar;
use reqwest::{self, Client, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

const URL: &str = "http://localhost:8080/api/rpc";
const LOGIN_URL: &str = "http://localhost:8080/api/login";

pub fn create_client_with_cookies() -> Client {
    let jar = Jar::default();
    Client::builder().cookie_store(true).build().unwrap()
}

pub async fn login(client: &Client) -> Result<()> {
    let req_create_user = json!({
    "username": "Dallas",
    "pwd": "hello"
    });

    let create = client
        .post("http://localhost:8080/api/create_user")
        .json(&req_create_user)
        .send()
        .await?;

    if create.status().is_success() {
    } else {
        println!("User create failed with status code: {}", create.status());
    }

    let req_login = json!({
    "username": "Dallas",
    "pwd": "hello"
    });

    let response = client.post(LOGIN_URL).json(&req_login).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        return Err(response.error_for_status().unwrap_err()).into();
    }
}

#[derive(Deserialize)]
pub struct RoomResponse {
    pub id: i64,
    pub result: Vec<Room>,
}

#[derive(Deserialize, Serialize)]
pub struct Room {
    pub id: i64,
    pub title: String,
}

#[derive(Deserialize)]
struct MessageResponse {
    id: i64,
    result: Vec<Message>,
}

#[derive(Deserialize, Serialize)]
pub struct Message {
    pub id: i64,
    pub message_text: String,
    pub message_room_id: i64,
    pub message_user_name: String,
}

pub async fn get_rooms(client: &Client) -> Vec<Room> {
    let req_list_rooms = json!({
        "id": 1,
        "method": "list_rooms"
    });

    let response = client.post(URL).json(&req_list_rooms).send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let body_text = resp.text().await.unwrap_or_else(|err| {
                eprintln!("Error reading response body: {:?}", err);
                String::new() // or handle this error case accordingly
            });

            //println!("Received JSON: {}", body_text);

            let room_response: RoomResponse =
                serde_json::from_str(&body_text).unwrap_or_else(|err| {
                    eprintln!("Error parsing room data: {:?}", err);
                    RoomResponse {
                        id: 0,
                        result: Vec::new(),
                    } // Return an empty response or handle this error case accordingly
                });

            room_response.result
        }
        Ok(resp) => {
            eprintln!("Room request failed with status code: {}", resp.status());
            Vec::new() // or handle this error case accordingly
        }
        Err(err) => {
            eprintln!("Error fetching rooms: {:?}", err);
            Vec::new() // or handle this error case accordingly
        }
    }
}

pub async fn get_messages(client: &Client, room_id: i64) -> Vec<Message> {
    let req_messages = json!({
        "id": 1,
        "method": "get_messages_by_room_id",
        "params": room_id + 1,
    });

    let response = client.post(URL).json(&req_messages).send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let body_text = resp.text().await.unwrap_or_else(|err| {
                eprintln!("Error reading response body: {:?}", err);
                String::new() // or handle this error case accordingly
            });

            //println!("Received JSON: {}", body_text);

            let message_response: MessageResponse = serde_json::from_str(&body_text)
                .unwrap_or_else(|err| {
                    eprintln!("Error parsing room data: {:?}", err);
                    MessageResponse {
                        id: 0,
                        result: Vec::new(),
                    } // Return an empty response or handle this error case accordingly
                });

            message_response.result
        }
        Ok(resp) => {
            eprintln!("Message request failed with status code: {}", resp.status());
            Vec::new() // or handle this error case accordingly
        }
        Err(err) => {
            eprintln!("Error fetching rooms: {:?}", err);
            Vec::new() // or handle this error case accordingly
        }
    }
}

pub async fn send_message(client: &Client, room_id: i64, text: Vec<String>) -> Result<()> {
    let message = text.join(" ");
    let send_message = json!({
                "method": "send_message",
                "params": {
                "data": {
                   "message_text": message,
                   "message_room_id": room_id + 1,
                   "message_user_name": "Dallas",
                }
            }
    });

    let response = client.post(URL).json(&send_message).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        return Err(response.error_for_status().unwrap_err()).into();
    }
}
