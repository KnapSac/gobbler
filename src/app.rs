use clap::{crate_name, crate_version, App, Arg, SubCommand};

pub fn build_app() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .arg(
            Arg::with_name("list")
                .long("list")
                .short("l")
                .help("List RSS feed subscriptions"),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add a RSS feed to subscribe to")
                .arg(
                    Arg::with_name("name")
                        .long("name")
                        .short("n")
                        .help("The name of the blog")
                        .required(true)
                        .takes_value(true)
                        .value_name("NAME"),
                )
                .arg(
                    Arg::with_name("url")
                        .long("url")
                        .short("u")
                        .help("The url of the blog's RSS feed")
                        .required(true)
                        .takes_value(true)
                        .value_name("URL"),
                ),
        )
}
