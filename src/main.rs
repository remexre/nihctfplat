use failure::{bail, Fallible};
use log::warn;
use nihctfplat::{
    dal::{Mailer, DB},
    router::serve_on,
    util::log_err,
};
use std::{
    net::{SocketAddr, ToSocketAddrs},
    process::exit,
};
use structopt::StructOpt;
use tokio::runtime::Builder;

fn main() {
    dotenv::dotenv().ok();

    let options = Options::from_args();
    if let Err(err) = options.start_logger() {
        warn!("Logging couldn't start: {}", err);
    }

    if let Err(err) = run(options) {
        log_err(&err);
        exit(1);
    }
}

fn run(options: Options) -> Fallible<()> {
    let serve_addr = options.serve_addr()?;
    let mut runtime = Builder::new().build()?;
    let db = DB::connect(&options.database_url)?;
    let smtp_from = options
        .smtp_from
        .as_ref()
        .unwrap_or(&options.smtp_user)
        .clone();
    let mailer = Mailer::connect(
        &options.smtp_host,
        !options.smtp_insecure,
        options.smtp_user,
        options.smtp_pass,
        smtp_from,
    )?;
    runtime.block_on(serve_on(serve_addr, db, mailer))
}

#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "::structopt::clap::AppSettings::ColoredHelp"))]
pub struct Options {
    /// Turns off message output. Passing once prevents logging to syslog. Passing twice or more
    /// disables all logging.
    #[structopt(short = "q", long = "quiet", parse(from_occurrences))]
    quiet: usize,

    /// Increases the verbosity. Default verbosity is warnings and higher to syslog, info and
    /// higher to the console.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// The URL of the Postgres database.
    #[structopt(long = "db", env = "DATABASE_URL")]
    pub database_url: String,

    /// The host to serve on.
    #[structopt(short = "H", long = "host", env = "HOST", default_value = "::")]
    host: String,

    /// The port to serve on.
    #[structopt(short = "P", long = "port", env = "PORT", default_value = "8080")]
    port: u16,

    /// The SMTP server's hostname.
    #[structopt(long = "smtp-host", env = "SMTP_HOST")]
    pub smtp_host: String,

    /// Whether to make SMTP less secure.
    #[structopt(long = "smtp-insecure")]
    pub smtp_insecure: bool,

    /// The user to authenticate to the SMTP server with. Usually your email address.
    #[structopt(long = "smtp-user", env = "SMTP_USER")]
    pub smtp_user: String,

    /// The password to authenticate to the SMTP server with.
    #[structopt(long = "smtp-pass", env = "SMTP_PASS")]
    pub smtp_pass: String,

    /// The From address for emails. Maybe be of the form "email@host.com" or
    /// "Foo Bar <email@host.com>". Defaults to the SMTP user.
    #[structopt(long = "smtp-from", env = "SMTP_FROM")]
    pub smtp_from: Option<String>,

    /// The syslog server to send logs to.
    #[structopt(short = "s", long = "syslog-server", env = "SYSLOG_SERVER")]
    syslog_server: Option<String>,
}

impl Options {
    /// Get the address to serve on.
    pub fn serve_addr(&self) -> Fallible<SocketAddr> {
        let addrs = (&self.host as &str, self.port)
            .to_socket_addrs()?
            .collect::<Vec<_>>();
        if addrs.is_empty() {
            bail!("No matching address exists")
        } else {
            Ok(addrs[0])
        }
    }

    /// Sets up logging as specified by the `-q`, `-s`, and `-v` flags.
    pub fn start_logger(&self) -> Fallible<()> {
        use fern::Dispatch;
        use log::LevelFilter;

        if self.quiet >= 2 {
            return Ok(());
        }

        let (console_ll, syslog_ll) = match self.verbose {
            0 => (LevelFilter::Info, LevelFilter::Warn),
            1 => (LevelFilter::Debug, LevelFilter::Info),
            2 => (LevelFilter::Trace, LevelFilter::Debug),
            _ => (LevelFilter::Trace, LevelFilter::Trace),
        };

        let fern = Dispatch::new().chain(
            Dispatch::new()
                .level(console_ll)
                .format(move |out, message, record| {
                    out.finish(format_args!("[{}] {}", record.level(), message))
                })
                .chain(std::io::stderr()),
        );

        let fern = if self.quiet == 0 {
            let formatter = syslog::Formatter3164 {
                facility: syslog::Facility::LOG_DAEMON,
                hostname: hostname::get_hostname(),
                process: "nihctfplat".to_owned(),
                pid: ::std::process::id() as i32,
            };

            let syslog = if let Some(ref server) = self.syslog_server {
                syslog::tcp(formatter, server).map_err(failure::SyncFailure::new)?
            } else {
                syslog::unix(formatter.clone())
                    .or_else(|_| syslog::tcp(formatter.clone(), ("127.0.0.1", 601)))
                    .or_else(|_| {
                        syslog::udp(formatter.clone(), ("127.0.0.1", 0), ("127.0.0.1", 514))
                    })
                    .map_err(failure::SyncFailure::new)?
            };

            fern.chain(Dispatch::new().level(syslog_ll).chain(syslog))
        } else {
            fern
        };

        fern.apply()?;
        Ok(())
    }
}
