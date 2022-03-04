use crate::DarnCommand;
use anyhow::Result;
use clap::{AppSettings, Arg, ArgGroup, ArgMatches, Command};
use daml::util::DamlSandboxTokenBuilder;

/// Darn command for generating Daml Sandbox auth tokens.
pub struct CommandToken {}

impl DarnCommand for CommandToken {
    fn name(&self) -> &str {
        "token"
    }

    fn args<'a, 'b>(&self) -> Command<'a> {
        args()
    }

    fn execute(&self, matches: &ArgMatches) -> Result<()> {
        execute(matches)
    }
}

enum TokenType {
    RS256,
    ES256,
}

enum TokenTimeToLive {
    Duration(i64),
    Expiry(i64),
}

enum OutputFormat {
    Token,
    Json,
    Both,
}

fn args<'a>() -> Command<'a> {
    Command::new("token")
        .about("Generate a Daml sandbox token")
        .setting(AppSettings::DeriveDisplayOrder)
        .arg(
            Arg::new("expiry")
                .long("expiry")
                .short('e')
                .takes_value(true)
                .value_name("timestamp")
                .help("Sets the token expiry time (unix timestamp)"),
        )
        .arg(
            Arg::new("duration")
                .long("duration")
                .short('d')
                .takes_value(true)
                .value_name("seconds")
                .help("Sets the duration of the token (seconds)"),
        )
        .arg(
            Arg::new("token-type")
                .long("token-type")
                .short('t')
                .takes_value(true)
                .possible_values(&["rs256", "es256"])
                .required(true)
                .help("Sets the token type"),
        )
        .arg(
            Arg::new("key-file")
                .long("key-file")
                .short('k')
                .takes_value(true)
                .value_name("filename")
                .required(true)
                .help("The file to use to sign the token"),
        )
        .arg(
            Arg::new("ledger-id")
                .long("ledger-id")
                .short('l')
                .takes_value(true)
                .required(true)
                .help("Sets the token ledgerId"),
        )
        .arg(
            Arg::new("participant-id")
                .long("participant-id")
                .short('P')
                .takes_value(true)
                .help("Sets the token participantId"),
        )
        .arg(
            Arg::new("application-id")
                .long("application-id")
                .short('A')
                .takes_value(true)
                .help("Sets the token applicationId"),
        )
        .arg(
            Arg::new("act-as")
                .long("act-as")
                .short('a')
                .multiple_values(true)
                .use_value_delimiter(true)
                .takes_value(true)
                .value_name("party")
                .help("Sets the token actAs list"),
        )
        .arg(
            Arg::new("read-as")
                .long("read-as")
                .short('r')
                .multiple_values(true)
                .use_value_delimiter(true)
                .takes_value(true)
                .value_name("party")
                .help("Sets the token readAs list"),
        )
        .arg(Arg::new("admin").long("admin").short('S').help("Sets the token admin flag"))
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .takes_value(true)
                .possible_values(&["token", "json", "both"])
                .default_value("token")
                .help("Sets the output format"),
        )
        .group(ArgGroup::new("ttl").args(&["expiry", "duration"]).required(true))
}

fn execute(matches: &ArgMatches) -> Result<()> {
    let admin = matches.is_present("admin");
    let act_as: Vec<String> = matches.values_of("act-as").unwrap_or_default().map(ToOwned::to_owned).collect();
    let read_as: Vec<String> = matches.values_of("read-as").unwrap_or_default().map(ToOwned::to_owned).collect();
    let ledger_id = matches.value_of("ledger-id").unwrap();
    let key_file = matches.value_of("key-file").unwrap();
    let participant_id = matches.value_of("participant-id");
    let application_id = matches.value_of("application-id");
    let token_ttl = match (matches.value_of("expiry"), matches.value_of("duration")) {
        (Some(expiry), None) => TokenTimeToLive::Expiry(expiry.parse::<i64>()?),
        (None, Some(duration)) => TokenTimeToLive::Duration(duration.parse::<i64>()?),
        _ => unreachable!(),
    };
    let token_type = match matches.value_of("token-type") {
        Some("rs256") => TokenType::RS256,
        Some("es256") => TokenType::ES256,
        _ => unreachable!(),
    };
    let format = match matches.value_of("output").unwrap() {
        "token" => OutputFormat::Token,
        "json" => OutputFormat::Json,
        "both" => OutputFormat::Both,
        _ => unreachable!(),
    };
    let mut token_builder = match token_ttl {
        TokenTimeToLive::Duration(duration) => DamlSandboxTokenBuilder::new_with_duration_secs(duration),
        TokenTimeToLive::Expiry(expiry) => DamlSandboxTokenBuilder::new_with_expiry(expiry),
    }
    .ledger_id(ledger_id)
    .admin(admin)
    .act_as(act_as)
    .read_as(read_as);
    if let Some(p) = participant_id {
        token_builder = token_builder.participant_id(p);
    }
    if let Some(a) = application_id {
        token_builder = token_builder.application_id(a);
    }
    let json = token_builder.claims_json()?;
    let token = match token_type {
        TokenType::RS256 => token_builder.new_rs256_token(std::fs::read_to_string(key_file)?)?,
        TokenType::ES256 => token_builder.new_ec256_token(std::fs::read_to_string(key_file)?)?,
    };
    match format {
        OutputFormat::Token => println!("{}", token),
        OutputFormat::Json => println!("{}", json),
        OutputFormat::Both => println!("json: {}\ntoken: {}", json, token),
    };
    Ok(())
}
