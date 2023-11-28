mod dns;
mod godaddy;
mod ip_addr;
mod models;

use random_word::Lang;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

use clap::{App, Arg, SubCommand};
use env_logger::Builder;
use log::LevelFilter;
use std::net::Ipv4Addr;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    Builder::new().filter(None, LevelFilter::Info).init();

    let api_key = match get_api_key() {
        Ok(k) => k,
        Err(e) => {
            println!("Error reading API key: {}", e);
            process::exit(1);
        }
    };
    let godaddy = godaddy::Godaddy::new(api_key);

    let matches = App::new("My DNS Manager")
        .version("1.0")
        .author("Your Name")
        .about("Manages DNS records via GoDaddy API")
        .subcommand(SubCommand::with_name("list").about("Lists all domains"))
        .subcommand(
            SubCommand::with_name("records")
                .about("Lists all records under a domain")
                .arg(
                    Arg::with_name("DOMAIN")
                        .help("The domain to list records for")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds an A record to a domain")
                .args(&[
                    Arg::with_name("DOMAIN")
                        .help("The domain to add the record to")
                        .required(true),
                    Arg::with_name("NAME")
                        .help("The subdomain name to add")
                        .long("name")
                        .takes_value(true)
                        .required(false),
                    Arg::with_name("IP")
                        .help("The IP address for the A record")
                        .long("ip")
                        .takes_value(true)
                        .required(false),
                    Arg::with_name("wait")
                        .help("Wait until the record has been successfully added")
                        .long("wait")
                        .takes_value(false), // No value, just the presence of the flag
                ]),
        )
        .get_matches();

    if let Some(_) = matches.subcommand_matches("list") {
        // Call function to list domains
        list_domains(&godaddy).await;
        process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("records") {
        if let Some(domain) = matches.value_of("DOMAIN") {
            // Call function to list records for the specified domain
            list_records_under_domain(&domain.to_string(), &godaddy).await;
            process::exit(0);
        }
    }

    if let Some(matches) = matches.subcommand_matches("add") {
        if let Some(domain) = matches.value_of("DOMAIN") {
            let sub_domain = matches
                .value_of("NAME")
                .unwrap_or_else(|| random_word::gen(Lang::En));

            let public_ip = ip_addr::get_public_ip().await;
            let ip = matches.value_of("IP").unwrap_or_else(|| public_ip.as_str());
            // Call function to add an A record to the specified domain
            println!("domain: {}, sub_domain: {}, ip: {}", domain, sub_domain, ip);
            godaddy
                .add_record(&domain.to_string(), &sub_domain.to_string(), ip.to_string())
                .await;

            let wait = matches.is_present("wait");
            if wait {
                let full_domain = format!("{}.{}", sub_domain, domain);
                let success =
                    wait_for_dns_propagation(&full_domain, ip, Duration::from_secs(300)).await;
                if success {
                    println!("Record added successfully: {}", full_domain);
                    process::exit(0);
                } else {
                    eprintln!("Record was not added successfully");
                    process::exit(1);
                }
            }
            process::exit(0);
        } else {
            println!("Error: Domain is required");
            process::exit(1);
        }
    }

    loop {
        display_menu();

        let choice = get_user_input();
        match choice.trim() {
            "1" => list_domains(&godaddy).await,
            "2" => {
                let selected_domain = choose_domain(&godaddy).await;
                list_records_under_domain(&selected_domain, &godaddy).await
            }
            "3" => add_a_record_under_domain(&godaddy).await,
            "q" => break,
            _ => println!("Invalid selection, choose again."),
        }
        println!();
    }
}

fn get_api_key() -> Result<String, std::io::Error> {
    if let Ok(api_key) = env::var("API_KEY") {
        return Ok(api_key.trim().to_string());
    }
    // If API_KEY environment variable is not set, read from the file
    fs::read_to_string("godaddy.conf").map(|s| s.trim().to_string())
}

fn display_menu() {
    println!("----- Menu -----");
    println!("1. List domains");
    println!("2. List records under domain");
    println!("3. Add A record under domain");
    println!("Type 'q' to quit");
    print!("Select: ");
    io::stdout().flush().unwrap();
}

fn get_user_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

async fn list_records_under_domain(selected_domain: &String, godaddy: &godaddy::Godaddy) {
    let records = godaddy.fetch_records_for_domain(&selected_domain).await;

    println!("{:<2}|{:<15}|{:<36}|{:<5}", "  ", "Name", "Data", "Type");
    println!("{:-<2}+{:-<15}+{:-<36}+{:-<5}", "", "", "", "");
    for (index, record) in records.iter().enumerate() {
        println!(
            "{:<2}|{:<15}|{:<36}|{:<5}",
            index + 1,
            record.name,
            record.data,
            record.record_type
        );
    }
}

async fn list_domains(godaddy: &godaddy::Godaddy) {
    let domains = godaddy.list_domains().await;
    println!("Your domains:");
    for (index, domain) in domains.iter().enumerate() {
        println!("{}. {}", index + 1, domain.domain);
    }
}

async fn choose_domain(godaddy: &godaddy::Godaddy) -> String {
    let domains = godaddy.list_domains().await;
    println!("\nChoose a domain[1]：");
    for (i, domain) in domains.iter().enumerate() {
        println!("{}. {}", i + 1, domain.domain);
    }
    io::stdout().flush().unwrap();

    let choice = get_user_input();
    let index = choice.trim().parse::<usize>().unwrap_or(1);

    if index >= 1 && index <= domains.len() {
        domains[index - 1].domain.clone()
    } else {
        domains[0].domain.clone()
    }
}

async fn add_a_record_under_domain(godaddy: &godaddy::Godaddy) {
    let selected_domain = choose_domain(&godaddy).await;
    println!("Add A record under {}", selected_domain);

    let rand_word = random_word::gen(Lang::En);
    print!("Input subdomain[{}]: ", rand_word);
    io::stdout().flush().unwrap();
    let mut subdomain = get_user_input().trim().to_string();

    if subdomain.is_empty() {
        subdomain = rand_word.to_string();
        println!("Use random subdomain: {}", subdomain);
    }

    let local_ip = ip_addr::get_public_ip().await;
    print!("Add record for IP [{}]:", local_ip);
    io::stdout().flush().expect("Should flush successfully");

    let target_ip = get_user_input();
    let record_ip = target_ip.trim().parse::<String>().unwrap_or(local_ip);
    godaddy
        .add_record(&selected_domain, &subdomain, record_ip.to_string())
        .await;
}

async fn wait_for_dns_propagation(domain: &str, expected_value: &str, timeout: Duration) -> bool {
    let start_time = Instant::now();
    let ip_addr: Ipv4Addr = expected_value.parse().unwrap();
    while start_time.elapsed() < timeout {
        match dns::check_dns_propagation(domain, ip_addr).await {
            Ok(true) => return true,
            _ => sleep(Duration::from_secs(30)), // Wait for 30 seconds before retrying
        }
    }
    false
}
