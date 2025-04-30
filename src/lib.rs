pub mod apartments;
pub mod app;
pub mod bookshelf;

use app::RunExercises;

use std::io;
use std::process::exit;

fn run_app(output: &mut impl io::Write, error_output: &mut impl io::Write) -> i32 {
    //-------------------------------------
    //Create the Application Object

    let mut app = RunExercises::new();

    //------------------------
    //Execute the Application

    let ierr = app.run(output, error_output);

    if app.options.verbosity > 1 {
        output
            .write_fmt(format_args!("App dmp:\n{:?}", app))
            .expect("STDOUT is closed!");
    }

    //------------------------
    //Build the Report

    if app.options.verbosity > 1 {
        if ierr == 0 {
            error_output
                .write_fmt(format_args!("Application finished with [{}]", ierr))
                .expect("STDERR is closed!");
        } else {
            error_output
                .write_fmt(format_args!("Application failed with [{}]", ierr))
                .expect("STDERR is closed!");
        }
    } //if app.options.verbosity > 0

    ierr
}

pub fn main() {
    let mut output = io::stdout();
    let mut error_output = io::stderr();

    let ierr = run_app(&mut output, &mut error_output);

    match ierr {
        0 => {}
        _ => {
            exit(ierr);
        }
    }
}
