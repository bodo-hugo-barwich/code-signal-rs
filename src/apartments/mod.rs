use crate::app::AppOptions;
use crate::app::Commands;

pub mod model;

use model::{Apartment, Building, Floor};

pub fn main(options: &AppOptions) -> i32 {
    let mut building: Building = Building::from_file();
    let mut ierr = 0;

    if building.floors.len() == 0 {
        // Create Apartments
        // 2 floors with each one 3 apartments
        for floor_idx in 1..=2 {
            let mut floor: Floor = Default::default();

            floor.number = floor_idx;

            for door_letter in ["A", "B", "C"] {
                let apartment =
                    Apartment::from_code(format!("{}{}", floor_idx, door_letter), false);

                floor.apartments.push(apartment);
            }

            building.floors.push(floor);
        }
    }

    // Print debugging information
    if options.verbosity > 1 {
        println!("building 0 dmp: {:?}", building);
    }

    if let Some(Commands::Apartments { occupy, .. }) = &options.command {
        match occupy {
            Some(apt) => {
                let mut apt_match: Option<&Apartment> = None;

                for floor in &mut building.floors {
                    for apartment in &mut floor.apartments {
                        if apartment.code == *apt {
                            if options.verbosity > 0 {
                                println!("Apartment '{}': Apartment occupied now.", &apt);
                            }

                            apartment.occupied = true;

                            apt_match = Some(apartment);
                        }
                    }
                }

                if apt_match.is_none() {
                    if options.verbosity > 0 {
                        eprintln!("Apartment '{}': Apartment does not exist!", &apt);
                    }

                    ierr = 1;
                }
            }
            None => {}
        }
    }

    if let Some(Commands::Apartments { list, .. }) = &options.command {
        if *list {
            println!("Building: Printing Apartments ...");

            // List Apartments
            for floor in &building.floors {
                println!("Floor No. {}:", floor.number);

                for apartment in &floor.apartments {
                    print!("{:?}, ", apartment);
                }

                println!("");
            }
        }
    }

    match building.to_file() {
        Ok(()) => {
            println!("Building: Configuration saved.");
        }
        Err(e) => {
            eprintln!("Building: Configuration save failed: {:?}", e);
            ierr = 1;
        }
    }

    ierr
}
