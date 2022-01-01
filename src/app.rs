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
        .subcommand(
            App::new("add")
                .about("Add a RSS feed to subscribe to")
                .arg(
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .help("The name of the blog")
                        .required(true)
                        .takes_value(true)
                        .value_name("NAME"),
                )
                .arg(
                    Arg::new("url")
                        .long("url")
                        .short('u')
                        .help("The url of the blog's RSS feed")
                        .required(true)
                        .takes_value(true)
                        .value_name("URL"),
                ),
        )
}
