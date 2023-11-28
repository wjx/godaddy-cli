use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Domain {
    pub domain: String,
}

#[derive(Deserialize)]
pub struct Record {
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub data: String,
    // add other fields if needed
}

#[derive(Debug, Serialize)]
pub struct DnsRecord {
    pub data: String, // The IP address
    pub ttl: u32,
}
