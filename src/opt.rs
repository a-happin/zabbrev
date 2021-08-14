use structopt::{clap, StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(
    name = clap::crate_name!(),
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
    about = clap::crate_description!(),
    version_short = "v",
    setting(clap::AppSettings::ColoredHelp),
)]
pub struct Opt {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

impl Opt {
    pub fn parse() -> Self {
        Self::from_args()
    }
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    #[structopt(about = "Initialize the plugin")]
    Init(InitArgs),

    #[structopt(about = "Expand abbreviation")]
    Expand(ExpandArgs),
}

#[derive(Debug, StructOpt)]
pub struct InitArgs {}

#[derive(Debug, StructOpt)]
pub struct ExpandArgs {
    #[structopt(help = "$LBUFFER", long = "lbuffer", short = "l")]
    pub lbuffer: String,

    #[structopt(help = "$RBUFFER", long = "rbuffer", short = "r")]
    pub rbuffer: String,
}
