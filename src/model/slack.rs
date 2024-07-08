use serde::Serialize;

#[derive(Serialize, PartialEq, Debug)]
pub struct SlackMessage {
    pub r#type: String,
    pub text: Text,
    pub accessory: Accessory,
}

#[derive(Serialize, PartialEq, Debug)]
pub struct Text {
    pub r#type: String,
    pub text: String,
}

#[derive(Serialize, PartialEq, Debug)]
pub struct Accessory {
    pub r#type: String,
    pub image_url: String,
    pub alt_text: String,
}

