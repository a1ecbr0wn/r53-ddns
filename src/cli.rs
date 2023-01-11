use clap::Parser;

/// Struct containing the parsed command line arguments
#[derive(Parser)]
#[command(name = "r53-ddns")]
#[command(bin_name = "r53-ddns")]
#[command(author, version, about, long_about = None, arg_required_else_help(true), disable_version_flag(true))]
pub struct Options {
    /// the server to save
    #[arg(short, long, value_parser, display_order(1))]
    pub server: Option<String>,

    /// the domain to save the record in
    #[arg(short, long, value_parser, display_order(2))]
    pub domain: Option<String>,

    /// the record is a nat router and so a *.server CNAME record will be set
    #[arg(short, long, value_parser, display_order(3))]
    pub nat: bool,

    /// verbose logging
    #[arg(short, long, value_parser, display_order(4), default_value_t = false)]
    pub verbose: bool,

    /// check frequency in seconds for continuous checking
    #[arg(short, long, value_parser, display_order(5), default_value_t = 0)]
    pub check: u64,

    /// print version information
    #[arg(short = 'V', long, value_parser, display_order(6))]
    pub version: bool,
}
