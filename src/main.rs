mod cli;

use std::env;
use std::net::IpAddr;
use std::path::Path;
use std::process::{Command, ExitStatus};
use std::str::FromStr;

use clap::Parser;
use futures::stream::{FuturesUnordered, StreamExt};
use lazy_static::lazy_static;
use log::{debug, error, info, warn, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use rand::seq::IteratorRandom;
use regex::Regex;
use reqwest::Client;
use rusoto_core::{Region, RusotoError};
use rusoto_route53::{
    Change, ChangeBatch, ChangeResourceRecordSetsRequest, ListHostedZonesRequest,
    ListResourceRecordSetsRequest, ResourceRecord, ResourceRecordSet, Route53, Route53Client,
};
use snapcraft::check_snap_home;
use tokio::{
    join, select,
    time::{sleep, Duration},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<RusotoError<RusotoError<()>>>> {
    lazy_static! {
        static ref DESCRIPTION: String = format!(env!("CARGO_PKG_VERSION"));
    };

    let options = cli::Options::parse();
    let check_freq = options.check;

    // set up logging
    let log_dir = Path::new(&options.logdir);
    let (log_file, log_roll) = if log_dir.is_dir() {
        let mut log_file = log_dir.to_path_buf();
        log_file.push("r53-ddns.log");
        let mut log_roll = log_dir.to_path_buf();
        log_roll.push("r53-ddns.{}.log");
        (
            log_file.display().to_string(),
            log_roll.display().to_string(),
        )
    } else {
        println!(
            "`{}' is not a valid path for the log file",
            log_dir.display()
        );
        (
            "/var/tmp/r53-ddns.log".to_string(),
            "/var/tmp/r53-ddns.{}.log".to_string(),
        )
    };

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build();
    let rolling_policy = CompoundPolicy::new(
        Box::new(SizeTrigger::new(1024 * 512)), // 512kb
        Box::new(FixedWindowRoller::builder().build(&log_roll, 4).unwrap()),
    );
    if let Ok(to_file) = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(&log_file, Box::new(rolling_policy))
    {
        let console_level = if options.debug {
            LevelFilter::Debug
        } else if options.verbose {
            LevelFilter::Info
        } else {
            LevelFilter::Warn
        };
        if let Ok(config) = Config::builder()
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(console_level)))
                    .build("stdout", Box::new(stdout)),
            )
            .appender(Appender::builder().build("to_file", Box::new(to_file)))
            .build(
                Root::builder()
                    .appender("stdout")
                    .appender("to_file")
                    .build(LevelFilter::Info),
            )
        {
            let _handle = log4rs::init_config(config).unwrap();

            let alert_script: String = if let Some(alert_script) = options.alert_script {
                let alert_script_path = Path::new(&alert_script);
                if alert_script_path.is_file() {
                    alert_script
                } else {
                    println!("invalid alert script supplied: {alert_script}");
                    "".to_string()
                }
            } else {
                println!("no alert script");
                "".to_string()
            };

            if env::var("AWS_SHARED_CREDENTIALS_FILE").is_err() {
                // if we are in a snap, rusoto will fail to read the credentials file from the $HOME/.aws/credential,
                // so set up that path but pointing to the real home rather than the snap home
                let (in_snap, home) = check_snap_home();
                if in_snap {
                    if let Some(mut credentials_file) = home {
                        credentials_file.push(".aws");
                        credentials_file.push("credentials");
                        if credentials_file.exists() {
                            if let Some(credentials_file) = credentials_file.as_path().to_str() {
                                unsafe {
                                    env::set_var("AWS_SHARED_CREDENTIALS_FILE", credentials_file);
                                }
                            }
                            debug!(
                                "within snap, AWS_SHARED_CREDENTIALS_FILE set to {credentials_file:?}",
                            );
                        }
                    }
                }
            }

            // Get the options for the ddns check, and run it
            if options.subdomain.is_some() && options.domain.is_some() {
                let mut subdomain_name = options.subdomain.unwrap();
                let mut zone_name = options.domain.unwrap();
                if is_valid_hostname(&subdomain_name) {
                    if !subdomain_name.ends_with('.') {
                        subdomain_name += ".";
                    }
                    if !zone_name.ends_with('.') {
                        zone_name += ".";
                    }
                    info!("subdomain:     {}", subdomain_name.clone());
                    info!("domain:        {}", zone_name.clone());
                    let region = match Region::from_str(options.region.as_str()) {
                        Ok(region) => region,
                        Err(_) => Region::UsEast1,
                    };
                    info!("region:        {}", region.name());
                    let client = Route53Client::new(region.clone());
                    let zone_id = get_zone_id(&client, &zone_name).await;
                    info!("zone id:       {zone_id}");

                    let nat = options.nat;
                    let ipaddresses: Option<Vec<String>> = options
                        .ipaddress_svc
                        .map(|addrs| addrs.split(',').map(|x| x.to_string()).collect());

                    if check_freq == 0 {
                        ddns_check(
                            &client,
                            &zone_id,
                            &zone_name,
                            &subdomain_name,
                            &ipaddresses,
                            nat,
                            &alert_script,
                        )
                        .await;
                    } else {
                        loop {
                            let client = Route53Client::new(region.clone());
                            ddns_check(
                                &client,
                                &zone_id,
                                &zone_name,
                                &subdomain_name,
                                &ipaddresses,
                                nat,
                                &alert_script,
                            )
                            .await;
                            sleep(Duration::from_millis(1000 * check_freq)).await;
                        }
                    }
                } else {
                    let err_msg = format!("invalid subdomain value: {subdomain_name}");
                    warn!("{err_msg}\n");
                    if !alert_script.is_empty() {
                        let msg = format!("{{ \"type\": \"error\", \"msg\": \"{err_msg}\" }}");
                        let _ = call_alert_script(&alert_script, &msg);
                    }
                }
            } else if options.subdomain.is_some() || options.domain.is_some() {
                println!("r53-ddns v{}\n", DESCRIPTION.as_str());
                error!("subdomain and domain parameters need to be supplied together");
                return Ok(());
            }
        } else {
            println!("Failed to create the log4rs configuration");
        }
    } else {
        println!(
            "Failed to create the rolling log file appender {}",
            &log_file
        );
    }

    if options.version {
        println!("r53-ddns v{}", DESCRIPTION.as_str());
        return Ok(());
    }

    Ok(())
}

async fn ddns_check(
    client: &Route53Client,
    zone_id: &str,
    zone_name: &str,
    subdomain_name: &str,
    ipaddresses: &Option<Vec<String>>,
    nat: bool,
    alert_script: &str,
) {
    let dns_name = format!("{subdomain_name}{zone_name}");
    let external_ip_future = get_external_ip_address(ipaddresses);
    let dns_ip_future = get_dns_record(
        client,
        zone_id,
        zone_name,
        subdomain_name,
        "A",
        alert_script,
    );
    let (external_ip_address, dns_ip_address) = join!(external_ip_future, dns_ip_future);
    if let Some(dns_ip_address) = dns_ip_address {
        if dns_ip_address != external_ip_address
            && !dns_ip_address.is_empty()
            && !external_ip_address.is_empty()
        {
            warn!(
                "{dns_name} ip address has changed from {dns_ip_address} to {external_ip_address}"
            );
            if !alert_script.is_empty() {
                let msg = format!("{{ \"type\": \"ip-change\", \"dns\": \"{dns_name}\", \"old\": \"{dns_ip_address}\", \"new\": \"{external_ip_address}\" }}");
                let _ = call_alert_script(alert_script, &msg);
            }
            set_dns_record(
                client,
                zone_id,
                zone_name,
                subdomain_name,
                "A",
                &external_ip_address,
            )
            .await;
        }
    } else if !external_ip_address.is_empty() {
        warn!("{dns_name} ip address is {external_ip_address}");
        if !alert_script.is_empty() {
            let msg = format!("{{ \"type\": \"ip-new\", \"dns\": \"{dns_name}\",  \"new\": \"{external_ip_address}\" }}");
            let _ = call_alert_script(alert_script, &msg);
        }
        set_dns_record(
            client,
            zone_id,
            zone_name,
            subdomain_name,
            "A",
            &external_ip_address,
        )
        .await;
    }
    if nat {
        let nat_subdomain_name = "\\052.".to_string() + subdomain_name;
        let dns_nat_cname = get_dns_record(
            client,
            zone_id,
            zone_name,
            &nat_subdomain_name,
            "CNAME",
            alert_script,
        )
        .await;
        if dns_nat_cname.is_none()
            || (dns_nat_cname.is_some() && dns_nat_cname.clone().unwrap() != dns_name)
        {
            info!("{nat_subdomain_name}{zone_name} nat CNAME set to {dns_name}");
            set_dns_record(
                client,
                zone_id,
                zone_name,
                &nat_subdomain_name,
                "CNAME",
                dns_name.as_str(),
            )
            .await
        }
    }
}

/// This function checks to see whether the subdomain_name entered into the zone and cname tags is a valid subdomain_name.
fn is_valid_hostname(subdomain_name: &str) -> bool {
    if subdomain_name.len() > 255 {
        return false;
    }
    let re = Regex::new(r"^[a-zA-Z\d-]{1,63}$").unwrap();
    for s in subdomain_name.split('.') {
        if !re.is_match(s) && !s.is_empty() {
            return false;
        }
    }
    true
}

/// Returns the zone id for a given zone name
async fn get_zone_id(client: &Route53Client, zone_name: &str) -> String {
    let mut next_marker: Option<String> = None;
    loop {
        // Create a request to list all hosted zones
        let request = ListHostedZonesRequest {
            marker: next_marker,
            ..Default::default()
        };
        // Send the request and print the response
        if let Ok(response) = client.list_hosted_zones(request).await {
            for zone in response.hosted_zones {
                if zone.name == zone_name {
                    let zone_id = match zone.id.rsplit_once('/') {
                        Some(id) => id.1.to_string(),
                        None => zone.id,
                    };
                    return zone_id;
                }
            }
            if response.is_truncated {
                next_marker = response.next_marker;
            } else {
                return "".to_string();
            }
        } else {
            return "".to_string();
        }
    }
}

/// Get the external ip address from an external service
async fn get_external_ip_address(ipaddresses: &Option<Vec<String>>) -> String {
    let mut futures = FuturesUnordered::new();

    let default_ipaddresses: Vec<String> = [
        "id.a1ecbr0wn.com",
        "a.ident.me",
        "ifconfig.me/ip",
        "icanhazip.com",
        "myexternalip.com/raw",
        "ipecho.net/plain",
        "checkip.amazonaws.com",
        "myip.dnsomatic.com",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect();
    let ipaddresses: Vec<String> = match ipaddresses {
        Some(ipaddresses) => {
            if ipaddresses.len() > 1 {
                ipaddresses
                    .iter()
                    .choose_multiple(&mut rand::rng(), 2)
                    .iter()
                    .map(|x| x.to_string())
                    .collect()
            } else {
                default_ipaddresses
                    .iter()
                    .choose_multiple(&mut rand::rng(), 2)
                    .iter()
                    .map(|x| x.to_string())
                    .collect()
            }
        }
        None => default_ipaddresses
            .iter()
            .choose_multiple(&mut rand::rng(), 2)
            .iter()
            .map(|x| x.to_string())
            .collect(),
    };

    // let addresses_fut = ipaddresses.into_iter().map(|x| get_http_resp(x.as_str()));
    for ipaddress in ipaddresses {
        let address_fut = get_http_resp(ipaddress);
        futures.push(address_fut);
    }
    for _ in 0..futures.len() {
        select! {
            response = futures.select_next_some() => {
                if let Ok((external_ip_address, external_address_svc)) = response {
                    info!("external ip:   {external_ip_address}  (from {external_address_svc})");
                    return external_ip_address;
                }
            }
        }
    }
    "".to_string()
}

async fn get_http_resp(address: String) -> Result<(String, String), ()> {
    let client = Client::new();
    if let Ok(resp) = client.get(format!("https://{address}")).send().await {
        if let Ok(response) = resp.text().await {
            let response = response.trim().to_string();
            if response.parse::<IpAddr>().is_ok() {
                return Ok((response, address.to_string()));
            }
        }
    }
    Err(())
}

//////////////////////////////
// Amazon Route 53 Interaction
//////////////////////////////

/// Lists the ip address of a given zone/host A record
async fn get_dns_record(
    client: &Route53Client,
    zone_id: &str,
    zone_name: &str,
    subdomain_name: &str,
    record_type: &str,
    alert_script: &str,
) -> Option<String> {
    let dns_name = format!("{subdomain_name}{zone_name}");
    let request = ListResourceRecordSetsRequest {
        hosted_zone_id: zone_id.to_string(),
        start_record_name: Some(dns_name.clone()),
        ..Default::default()
    };
    match client.list_resource_record_sets(request).await {
        Ok(response) => {
            let record_sets = response.resource_record_sets;
            for record_set in record_sets {
                if record_set.name == dns_name && record_set.type_ == *record_type {
                    if let Some(records) = &record_set.resource_records {
                        if let Some(record) = records.first() {
                            let ip_record = record.value.clone();
                            if record_type == "A" {
                                if ip_record.parse::<IpAddr>().is_ok() {
                                    info!("dns ip:        {ip_record}");
                                    return Some(ip_record);
                                }
                            } else {
                                info!("dns ip:        {ip_record}");
                                return Some(ip_record);
                            }
                        }
                    }
                }
            }
            debug!("No record for {dns_name} currently set up in Route 53")
        }
        Err(x) => {
            let credentials_file =
                if let Ok(credentials_file) = env::var("AWS_SHARED_CREDENTIALS_FILE") {
                    credentials_file
                } else {
                    "not found".to_string()
                };
            let err_msg = format!(
                "Unable to retrieve the current dns address for {dns_name}: {x} Home={}",
                credentials_file
            );
            warn!("{err_msg}");
            if !alert_script.is_empty() {
                let msg = format!("{{ : \"error\", \"msg\": \"{err_msg}\" }}");
                let _ = call_alert_script(alert_script, &msg);
            }
        }
    }
    None
}

/// Creates resource records in the given hosted zone
async fn set_dns_record(
    client: &Route53Client,
    zone_id: &str,
    zone_name: &str,
    subdomain_name: &str,
    record_type: &str,
    record_value: &str,
) {
    // Build the request to change the resource record sets
    let dns_name = format!("{subdomain_name}{zone_name}");
    let request = ChangeResourceRecordSetsRequest {
        hosted_zone_id: zone_id.to_string(),
        change_batch: ChangeBatch {
            changes: vec![Change {
                action: String::from("UPSERT"),
                resource_record_set: ResourceRecordSet {
                    name: dns_name,
                    resource_records: Some(vec![ResourceRecord {
                        value: record_value.to_string(),
                    }]),
                    ttl: Some(300),
                    type_: record_type.to_string(),
                    ..Default::default()
                },
            }],
            comment: Some(String::from("Updated by r53-ddns")),
        },
    };

    // Call the Route53 API to change the resource record sets
    let _response = client.change_resource_record_sets(request).await;
}

pub fn call_alert_script(script: &str, message: &str) -> std::io::Result<ExitStatus> {
    Command::new(script).args([message]).spawn()?.wait()
}
