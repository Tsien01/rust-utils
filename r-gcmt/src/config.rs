use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(rename="ticketNumber")]
    pub ticket_number: String,
    pub model: String,
}