use clap::Parser;

/// Struct containing the parsed command line arguments
#[derive(Parser)]
#[command(name = "r53-ddns")]
#[command(bin_name = "r53-ddns")]
#[command(author, version, about, long_about = None, arg_required_else_help(true), disable_version_flag(true))]
pub struct Options {
    /// The subdomain to save (required)
    #[arg(short, long, value_parser, display_order(1))]
    pub subdomain: Option<String>,

    /// The domain to save the record in  (required)
    #[arg(short, long, value_parser, display_order(2))]
    pub domain: Option<String>,

    /// The aws region, [default: us-east-1]
    #[arg(short, long, value_parser, display_order(3))]
    pub region: Option<String>,

    /// The ip address services to use, e.g. ident.me,ifconfig.me/ip
    #[arg(short, long, value_parser, display_order(4))]
    pub ipaddress_svc: Option<String>,

    /// The record is a nat router and so a *.<subdomain>.<domain> CNAME record will be set
    #[arg(short, long, value_parser, display_order(5))]
    pub nat: bool,

    /// Verbose logging
    #[arg(short, long, value_parser, display_order(6), default_value_t = false)]
    pub verbose: bool,

    /// Debug logging
    #[arg(short = 'D', long, value_parser, hide(true), default_value_t = false)]
    pub debug: bool,

    /// Consecutive check gap in seconds for continuous checking
    #[arg(short, long, value_parser, display_order(7), default_value_t = 0)]
    pub check: u64,

    /// Print version information
    #[arg(short = 'V', long, value_parser, display_order(8))]
    pub version: bool,
}
