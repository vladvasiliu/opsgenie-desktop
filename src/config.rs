use clap::{app_from_crate, AppSettings, Arg};
use url::Url;

pub struct Config {
    pub api_key: String,
    pub history_days: u8,
    pub update_interval: u8,
    pub request_limit: u8,
    pub base_path: Url,
}

impl Config {
    pub(crate) fn from_clap() -> Self {
        let matches = app_from_crate!()
            .setting(AppSettings::ColoredHelp)
            .arg(
                Arg::new("api-key")
                    .short('k')
                    .long("api-key")
                    .value_name("API_KEY")
                    .about("OpsGenie API Key")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::new("history-days")
                    .long("history")
                    .value_name("DAYS")
                    .about("Oldest closed alert to retrieve")
                    .takes_value(true)
                    .required(false)
                    .default_value("7"),
            )
            .arg(
                Arg::new("update-interval")
                    .short('i')
                    .long("interval")
                    .value_name("SECONDS")
                    .about("How long to wait between queries in seconds")
                    .takes_value(true)
                    .required(false)
                    .default_value("60"),
            )
            .arg(
                Arg::new("request-limit")
                    .long("request-limit")
                    .value_name("COUNT")
                    .about("Number of alerts per request")
                    .takes_value(true)
                    .required(false)
                    .default_value("100"),
            )
            .arg(
                Arg::new("base-path")
                    .long("base-path")
                    .value_name("URL")
                    .about("OpsGenie API base path to use")
                    .takes_value(true)
                    .required(false)
                    .default_value("https://api.opsgenie.com"),
            )
            .get_matches();
        Config {
            api_key: matches.value_of_t_or_exit("api-key"),
            history_days: matches.value_of_t_or_exit("history-days"),
            update_interval: matches.value_of_t_or_exit("update-interval"),
            request_limit: matches.value_of_t_or_exit("request-limit"),
            base_path: matches.value_of_t_or_exit("base-path"),
        }
    }
}
