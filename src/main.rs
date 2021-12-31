use ::log::{info, LevelFilter};
use anyhow::{anyhow, Context, Result};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::net::{Ipv4Addr, Ipv6Addr};

mod cloudflare;
mod ip;
mod log;

const API_TOKEN_ENV_VAR: &str = "API_TOKEN";
const DOMAIN_ENV_VAR: &str = "DOMAIN";
const FILE_CACHE_ENV_VAR: &str = "FILE_CACHE";
const FILE_DEFAULT: &str = "/tmp/cloudflare-ddns.txt";
const IPV4_TYPE: &str = "A";
const IPV6_TYPE: &str = "AAAA";

fn main() -> Result<()> {
    log::init(LevelFilter::Info).context("Failed to initialize logger")?;

    let ipv4_address = ip::query::<Ipv4Addr>()
        .context("Failed to get IPv4 address")?
        .to_string();

    let ipv6_address = ip::query::<Ipv6Addr>()
        .context("Failed to get IPv6 address")?
        .to_string();

    let token = env::var(API_TOKEN_ENV_VAR).context("Failed to find env API_TOKEN")?;
    let domain = env::var(DOMAIN_ENV_VAR).context("Failed to find env DOMAIN")?;

    let file_cache = env::var(FILE_CACHE_ENV_VAR).unwrap_or(FILE_DEFAULT.to_string());

    // try to compare cache with current ip address and exit earlier if ip equals cache
    match File::open(&file_cache) {
        Ok(mut f) => {
            let mut contents = String::new();
            f.read_to_string(&mut contents)?;
            if contents == ipv4_address {
                info!("Cached ip address matches current: {}", ipv4_address);
                return Ok(());
            }
        }
        Err(_) => {}
    };

    // Determine zone based on provided domain, only one level addresses supported
    let zone_name = domain.split(".").collect::<Vec<&str>>()[1..].join(".");

    let client = cloudflare::Client::new(token);

    let zone = client
        .zones(&zone_name)
        .context("Failed to get zone")?
        .into_iter()
        .nth(0)
        .ok_or(anyhow!("Zone not found"))?;

    let mut created = false;

    let dns_records = client
        .dns_records(&zone.id, &domain)
        .context(format!("Failed to get DNS records for domain {}", domain))?;

    // Create DNS records if they are missing
    if dns_records.len() == 0 {
        client
            .create_dns_record(&zone.id, &domain, &ipv4_address, IPV4_TYPE)
            .context(format!("Failed to create DNS record {}", domain))?;
        info!("Created record {} with value {}", domain, ipv4_address);
        client
            .create_dns_record(&zone.id, &domain, &ipv6_address, IPV6_TYPE)
            .context(format!("Failed to create DNS record {}", domain))?;
        info!("Created record {} with value {}", domain, ipv6_address);
        created = true;
    }

    // No need to proceed further if we just created new records
    if !created {
        for dns_record in dns_records {
            let ip_address = match dns_record.type_.as_str() {
                "A" => ipv4_address.clone(),
                "AAAA" => ipv6_address.clone(),
                _ => continue,
            };

            if dns_record.content == ip_address {
                info!(
                    "No update required for {} record ({})",
                    dns_record.type_, dns_record.content
                );
                continue;
            }

            client
                .patch_dns_record(&zone.id, &dns_record.id, &ip_address)
                .with_context(|| {
                    format!(
                        "Failed to update {} record from {} to {}",
                        dns_record.type_, dns_record.content, ip_address,
                    )
                })?;

            info!(
                "Updated {} record from {} to {}",
                dns_record.type_, dns_record.content, ip_address
            );
        }
    }

    // Write file cache
    let mut cache = File::create(&file_cache)?;
    cache.write_all(&ipv4_address.into_bytes())?;

    Ok(())
}
