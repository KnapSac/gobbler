use clap::{crate_name, crate_version, App, Arg};

pub fn build_app() -> App<'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .arg(
            Arg::new("list")
                .long("list")
                .short('l')
                .help("List RSS feed subscriptions"),
        )
        .arg(
            Arg::new("skip-empty-feeds")
                .long("skip-empty-feeds")
                .short('s')
                .help("Skip feeds which no items"),
        )
        .subcommand(
            App::new("add")
                .about("Add a RSS feed subscription")
                .arg(
                    Arg::new("name")
                        .help("The name of the blog")
                        .required(true)
                        .takes_value(true)
                        .value_name("NAME"),
                )
                .arg(
                    Arg::new("url")
                        .help("The url of the blog's RSS feed")
                        .required(true)
                        .takes_value(true)
                        .value_name("URL"),
                ),
        )
        .subcommand(
            App::new("remove")
                .about("Remove a RSS feed subscription")
                .arg(
                    Arg::new("name")
                        .help("The name of the blog")
                        .required(true)
                        .takes_value(true)
                        .value_name("NAME"),
                ),
        )
}
