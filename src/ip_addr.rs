use reqwest::header;

pub async fn get_public_ip() -> String {
    let client = reqwest::Client::builder()
        .http1_only()
        .http1_title_case_headers()
        // .redirect(redirect::Policy::none())
        .build()
        .unwrap();
    return client
        .get("http://ifconfig.me")
        .header(header::USER_AGENT, "curl/7.82.0")
        .header(header::HOST, "ifconfig.me")
        .header(header::ACCEPT, "*/*")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
}
