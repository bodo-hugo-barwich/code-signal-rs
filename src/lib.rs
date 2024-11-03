pub mod apartments;
pub mod app;
pub mod bookshelf;

use app::RunExercises;

use std::process::exit;

fn run_app() -> i32 {
    //-------------------------------------
    //Create the Application Object

    let mut app = RunExercises::new();

    //------------------------
    //Execute the Application

    let ierr = app.run();

    if app.options.verbosity > 1 {
        println!("App dmp:\n{:?}", app);
    }

    //------------------------
    //Build the Report

    if app.options.verbosity > 1 {
        if ierr == 0 {
            eprintln!("Application finished with [{}]", ierr);
        } else {
            eprintln!("Application failed with [{}]", ierr);
        }
    } //if app.options.verbosity > 0

    ierr
}

pub fn main() {
    let ierr = run_app();

    match ierr {
        0 => {}
        _ => {
            exit(ierr);
        }
    }
}
