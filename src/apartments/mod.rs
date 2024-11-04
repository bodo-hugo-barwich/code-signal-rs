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
                let apartment = Apartment::from_floor_door(floor_idx, door_letter, false);

                floor.apartments.push(apartment);
            }

            building.floors.push(floor);
        }
    }

    // Print debugging information
    if options.verbosity > 1 {
        println!("building 0 dmp: {:?}", building);
    }

    if let Some(Commands::Apartments { add, .. }) = &options.command {
        match add {
            Some(apt) => {
                let apt_search = Apartment::from_code(&*apt, false);

                if options.verbosity > 0 {
                    println!(
                        "Apartment '{}': Add Apartment {:?}",
                        &apt_search.code, apt_search
                    );
                }

                let mut floor_match: Option<&mut Floor> = None;
                let mut apt_match: Option<&Apartment> = None;
                let mut floor_exists = false;

                for floor in &building.floors {
                    if floor.number == apt_search.floor {
                        floor_exists = true;
                        break;
                    }
                }

                if !floor_exists {
                    let mut floor: Floor = Default::default();

                    floor.number = apt_search.floor;

                    building.floors.push(floor);
                }

                building.floors.sort();

                for floor in &mut building.floors {
                    if floor.number == apt_search.floor {
                        floor_match = Some(floor);
                        break;
                    }
                }

                if let Some(f) = floor_match {
                    for apartment in &mut f.apartments {
                        if apartment.door == apt_search.door {
                            apt_match = Some(apartment);
                        }
                    }

                    match apt_match {
                        Some(_) => {
                            if options.verbosity > 0 {
                                eprintln!(
                                    "Apartment '{}': Apartment does already exist.",
                                    &apt_search.code
                                );
                            }

                            // Mark command as failed
                            ierr = 1;
                        }
                        None => {
                            if options.verbosity > 0 {
                                println!("Apartment '{}': Apartment was added.", &apt_search.code);
                            }

                            f.apartments.push(apt_search);
                        }
                    }

                    f.apartments.sort_by(|a, b| a.door.cmp(&b.door));
                }
            }
            None => {}
        }
    }

    if let Some(Commands::Apartments { occupy, .. }) = &options.command {
        match occupy {
            Some(apt) => {
                let apt_search = Apartment::from_code(&*apt, true);
                let mut apt_match: Option<&Apartment> = None;

                for floor in &mut building.floors {
                    if floor.number == apt_search.floor {
                        for apartment in &mut floor.apartments {
                            if apartment.door == apt_search.door {
                                if !apartment.occupied {
                                    if options.verbosity > 0 {
                                        println!(
                                            "Apartment '{}': Apartment occupied now.",
                                            apartment.code
                                        );
                                    }

                                    apartment.occupied = true;
                                } else {
                                    if options.verbosity > 0 {
                                        eprintln!(
                                            "Apartment '{}': Apartment is already occupied!",
                                            apartment.code
                                        );
                                    }

                                    // Mark command as failed
                                    ierr = 1;
                                }

                                apt_match = Some(apartment);

                                break;
                            }
                        }

                        break;
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
