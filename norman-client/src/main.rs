//use std::{env, process};

//use norman_client::UserOptions;

fn main() {
    // let user_args = UserOptions::new(env::args()).unwrap_or_else(|err| {
    //     eprintln!("Problem parsing arguments: {}", err);
    //     process::exit(1);
    // });
    let _guard = sentry::init("https://3d034496ffe8417f988b81f617ee032c@sentry.io/4616084");

    sentry::integrations::panic::register_panic_handler();

    panic!("CI and Sentry Integration Test");

}
