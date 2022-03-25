use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerStatus {
    #[serde(rename = "description")]
    pub description: Description,

    #[serde(rename = "favicon")]
    #[serde(default)]
    pub favicon: String,

    #[serde(rename = "players")]
    pub players: Players,

    #[serde(rename = "version")]
    pub version: Version,
}

#[derive(Serialize, Deserialize)]
pub struct Description {
    #[serde(rename = "text")]
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct Players {
    #[serde(rename = "max")]
    pub max: i64,

    #[serde(rename = "online")]
    pub online: i64,

    #[serde(rename = "sample")]
    #[serde(default)]
    pub sample: Vec<Sample>,
}

#[derive(Serialize, Deserialize)]
pub struct Sample {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Version {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "protocol")]
    pub protocol: i64,
}
