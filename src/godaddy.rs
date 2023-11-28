use crate::models::{DnsRecord, Domain, Record};

pub struct Godaddy {
    api_key: String,
}

impl Godaddy {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn list_domains(&self) -> Vec<Domain> {
        let url = "https://api.godaddy.com/v1/domains";
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .header("Authorization", format!("sso-key {}", self.api_key))
            .send()
            .await;

        match response {
            Ok(resp) => {
                let domains: Vec<Domain> = match resp.json().await {
                    Ok(d) => d,
                    Err(e) => {
                        println!("Error parsing response: {}", e);
                        return Vec::new(); // 返回空的 Vec
                    }
                };

                domains
            }
            Err(e) => {
                println!("Error fetching domains: {}", e);
                Vec::new() // 在错误分支上返回空的 Vec
            }
        }
    }

    pub async fn fetch_records_for_domain(&self, domain: &str) -> Vec<Record> {
        let url = format!("https://api.godaddy.com/v1/domains/{}/records", domain);
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Authorization", format!("sso-key {}", self.api_key))
            .send()
            .await;

        match response {
            Ok(resp) => match resp.json::<Vec<Record>>().await {
                Ok(records) => records,
                Err(e) => {
                    println!("Error parsing response: {}", e);
                    Vec::new()
                }
            },
            Err(e) => {
                println!("Error fetching records: {}", e);
                Vec::new()
            }
        }
    }

    pub async fn add_record(&self, selected_domain: &String, subdomain: &String, ip: String) {
        let client = reqwest::Client::new();
        let api_url = format!(
            "https://api.godaddy.com/v1/domains/{}/records/A/{}",
            selected_domain, subdomain
        );

        let new_record = DnsRecord {
            data: ip,
            ttl: 3600,
        };

        let response = client
            .put(&api_url)
            .header("Authorization", format!("sso-key {}", self.api_key))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&[new_record]).unwrap())
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    println!("成功创建A记录！");
                } else {
                    println!("创建A记录失败: {}", resp.text().await.unwrap_or_default());
                }
            }
            Err(e) => println!("请求失败: {}", e),
        }
    }
}
