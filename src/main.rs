use std::{io::{self, Write}};
use serde::Serialize;
use std::env;

fn get_menu_choice(options: &[&str])  -> usize {
    loop {
        for (i, option) in options.iter().enumerate() {
            println!("[{}] {}", i + 1, option);
        }
        print!("> ");
        io::stdout().flush().unwrap(); // puts > before input

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= options.len() => return n,
            _ => println!("Invalid choice, try again!\n"),
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let zone_id = env::var("ZONE_ID")?;
    let token = env::var("CLOUDFLARE_API_TOKEN")?;
    let client = reqwest::Client::new();

    let options = ["List DNS records",
        "Add a CNAME record",
        "Delete a CNAME record",
        "Quit"];

    println!("=====================================
    Cloudflare DNS record manager
=====================================");
    loop {
        let choice = get_menu_choice(&options);

        match choice {
            1 => println!("listing records..."),
            2 => {
                print!("Enter the subdomain (e. g. mc.example.com): ");
                io::stdout().flush().unwrap();
                let mut subdomain = String::new();
                io::stdin().read_line(&mut subdomain).unwrap();

                print!("Enter the target (e. g. example.com): ");
                io::stdout().flush().unwrap();
                let mut target = String::new();
                io::stdin().read_line(&mut target).unwrap();

                // TODO: comments for the dns record

                add_cname_record(
                    &client, 
                    &zone_id, 
                    &token, 
                    &subdomain.trim(),
                    &target.trim()
                ).await?;
            } 
            3 => println!("deleting a CNAME record..."),
            4 => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

#[derive(Serialize)]
struct NewDnsRecord {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

async fn add_cname_record(
    client: &reqwest::Client,
    zone_id: &str,
    token: &str,
    subdomain: &str,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id);

    let record = NewDnsRecord {
        record_type: "CNAME".to_string(),
        name: subdomain.to_string(),
        content: target.to_string(),
        ttl: 1, // 1 = automatic
        proxied: false, // for minecraft / other game servers / non http apps this should be off
    };

    let resp = client
        .post(&url)
        .bearer_auth(token)
        .json(&record)
        .send()
        .await?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await?;

    if status.is_success() {
        println!("Created record: {} -> {}", subdomain, target);
    } else {
        println!("Failed to create record: {:#?}", body["errors"]);
    }

    Ok(())
}