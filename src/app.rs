use clap::{Parser, Subcommand};

/// Code Signal Rust Exercises
#[derive(Parser, Default, Debug)]
#[command(version, about, long_about = None)]
pub struct AppOptions {
    /// Turn debugging information on (use multiple times to increase verbosity)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbosity: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// apartment building exercise
    Apartments {
        /// lists apartments
        #[arg(short, long)]
        list: bool,

        /// occupy apartment of given code
        #[arg(short, long)]
        occupy: Option<String>,

        /// occupy apartment of given code
        #[arg(short, long)]
        add: Option<String>,
    },
    /// bookshelf listing excercise
    Bookshelf {},
}

//==============================================================================
// Structure RunExercises Declaration

#[derive(Debug)]
pub struct RunExercises {
    pub options: AppOptions,
    pub exit_code: i32,
}

//==============================================================================
// Structure RunClientAccounting Implementation

impl Default for RunExercises {
    /*----------------------------------------------------------------------------
     * Default Constructor
     */

    fn default() -> Self {
        RunExercises::new()
    }
}

#[allow(dead_code)]
impl RunExercises {
    /*----------------------------------------------------------------------------
     * Constructors
     */

    pub fn new() -> RunExercises {
        RunExercises {
            options: Default::default(),
            exit_code: 0,
        }
    }

    /*----------------------------------------------------------------------------
     * Administration Methods
     */

    pub fn run(&mut self) -> i32 {
        self.options = AppOptions::parse();

        // You can see how many times a particular flag or argument occurred
        // Note, only flags can have multiple occurrences
        match self.options.verbosity {
            0 => {}
            1 => println!("Application runs with normal verbosity."),
            2 => println!("Application runs with detailled verbosity"),
            _ => println!("Don't be crazy"),
        }

        // You can check for the existence of subcommands, and if found use their
        // matches just as you would the top level cmd
        match &self.options.command {
            Some(Commands::Apartments { .. }) => {
                self.exit_code = crate::apartments::main(&self.options);
            }
            Some(Commands::Bookshelf {}) => {
                self.exit_code = crate::bookshelf::main();
            }
            None => {}
        }

        // Continued program logic goes here...

        self.exit_code
    }
}
