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
            Arg::new("hide-empty-feeds")
                .long("hide-empty-feeds")
                .short('i')
                .help("Hide feeds with no items"),
        )
        .arg(
            Arg::new("weeks")
                .long("weeks")
                .short('w')
                .help("Show posts from the last NUM weeks")
                .takes_value(true)
                .value_name("NUM")
                .default_value("4"),
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
