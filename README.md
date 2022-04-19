# gobbler
A command line RSS feed subscriptions checker.

This project is Windows only, as it makes use of the Windows-specific Syndication API.

## How to use
Use the `add` and `remove` sub-commands to add and remove RSS feed subscriptions. Your active
subscriptions can be viewed by using the `--list` flag.

### Controlling what is shown
The `--weeks` option can be used to control the number of weeks from which items are shown, this
defaults to 4 weeks.

Additionally, passing the `--hide-empty-feeds` flag will hide feeds with no items in the last number
of specified weeks.

### Use in shell profile
`gobbler` is designed to be usable as the greeting command in your shell, i.e. the command which
runs when your shell is started. Since you probably do not want to see the output every time you
start a new shell, there is an option `--run-days` which allows you to specify after how many days
you want to see the output again. By default this is after 1 day, so if you include `gobbler
--run-days` in your shell profile, when starting your shell for the first time on any given day, you
will see the new items in the RSS feeds you are subscribed to.

If you do not want to see this daily, you can also use `gobbler --run-days=7` to see it every week
(or after any other amount of days you like).

## License
Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
