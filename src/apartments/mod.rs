use std::io;
use std::path::PathBuf;

use crate::app::{AppOptions, Commands};

pub mod model;

use model::{Apartment, Building, Floor};

pub fn main(
    options: &AppOptions,
    output: &mut impl io::Write,
    error_output: &mut impl io::Write,
) -> i32 {
    let mut building: Building = match &options.file {
        Some(f) => match f.as_str() {
            "-" => Building::from_file(Some(options.verbosity)),
            _ => Building::from_custom_file(PathBuf::from(f).as_path(), Some(options.verbosity)),
        },
        None => Building::from_file(Some(options.verbosity)),
    };
    let print_yaml = if let Some(f) = &options.file {
        if f.as_str() == "-" {
            true
        } else {
            false
        }
    } else {
        false
    };
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
        if print_yaml {
            output
                .write_fmt(format_args!("# building 0 dmp: {:?}\n", building))
                .expect("STDOUT closed");
        } else {
            output
                .write_fmt(format_args!("building 0 dmp: {:?}\n", building))
                .expect("STDOUT closed");
        }
    }

    if let Some(Commands::Apartments { add, .. }) = &options.command {
        match add {
            Some(apt) => {
                let apt_search = Apartment::from_code(&*apt, false);

                if options.verbosity > 0 {
                    if print_yaml {
                        // Print as comment
                        output
                            .write_fmt(format_args!(
                                "# Apartment '{}': Add Apartment {:?}",
                                &apt_search.code, apt_search
                            ))
                            .expect("STDOUT closed");
                    } else {
                        output
                            .write_fmt(format_args!(
                                "Apartment '{}': Add Apartment {:?}",
                                &apt_search.code, apt_search
                            ))
                            .expect("STDOUT closed");
                    }
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
                                error_output
                                    .write_fmt(format_args!(
                                        "Apartment '{}': Apartment does already exist.",
                                        &apt_search.code
                                    ))
                                    .expect("STDERR closed");
                            }

                            // Mark command as failed
                            ierr = 1;
                        }
                        None => {
                            if options.verbosity > 0 {
                                if print_yaml {
                                    // Print as comment
                                    output
                                        .write_fmt(format_args!(
                                            "# Apartment '{}': Apartment was added.",
                                            &apt_search.code
                                        ))
                                        .expect("STDOUT closed");
                                } else {
                                    output
                                        .write_fmt(format_args!(
                                            "Apartment '{}': Apartment was added.",
                                            &apt_search.code
                                        ))
                                        .expect("STDOUT closed");
                                }
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
                                        if print_yaml {
                                            output
                                                .write_fmt(format_args!(
                                                    "# Apartment '{}': Apartment occupied now.\n",
                                                    apartment.code
                                                ))
                                                .expect("STDOUT closed");
                                        } else {
                                            output
                                                .write_fmt(format_args!(
                                                    "Apartment '{}': Apartment occupied now.\n",
                                                    apartment.code
                                                ))
                                                .expect("STDOUT closed");
                                        }
                                    }

                                    apartment.occupied = true;
                                } else {
                                    if options.verbosity > 0 {
                                        if print_yaml {
                                            output
                                            .write_fmt(format_args!(
                                                "# Apartment '{}': Apartment is already occupied!\n",
                                                apartment.code
                                            ))
                                            .expect("STDOUT closed");
                                        } else {
                                            output
                                                .write_fmt(format_args!(
												        "Apartment '{}': Apartment is already occupied!\n",
												        apartment.code
												    ))
                                                .expect("STDOUT closed");
                                        }
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
                        if print_yaml {
                            error_output
                                .write_fmt(format_args!(
                                    "# Apartment '{}': Apartment does not exist!\n",
                                    &apt
                                ))
                                .expect("STDERR closed");
                        } else {
                            error_output
                                .write_fmt(format_args!(
                                    "Apartment '{}': Apartment does not exist!\n",
                                    &apt
                                ))
                                .expect("STDERR closed");
                        }
                    }

                    // Mark command as failed
                    ierr = 1;
                }
            }
            None => {}
        }
    }

    if let Some(Commands::Apartments { list, .. }) = &options.command {
        if *list {
            if options.verbosity > 1 {
                output
                    .write_fmt(format_args!("Building: Printing Apartments ...\n"))
                    .expect("STDOUT closed");
            }

            // List Apartments
            for floor in &building.floors {
                output
                    .write_fmt(format_args!("Floor No. {}:\n", floor.number))
                    .expect("STDOUT closed");

                for apartment in &floor.apartments {
                    output
                        .write_fmt(format_args!("{:?}, ", apartment))
                        .expect("STDOUT closed");
                }

                output.write_all("\n".as_bytes()).expect("STDOUT closed");
            }
        }
    }

    match &options.file {
        Some(f) => {
            if f == "-" {
                let _ = building.print(output).map_err(|e| {
                    error_output
                        .write_fmt(format_args!(
                            "Building: Configuration print failed: {:?}",
                            e
                        ))
                        .expect("STDERR closed");
                    // Mark command as failed
                    ierr = 1;
                });
            } else {
                match building.to_custom_file(PathBuf::from(f).as_path()) {
                    Ok(()) => {
                        output
                            .write_all("Building: Configuration saved.".as_bytes())
                            .expect("STDOUT closed");
                    }
                    Err(e) => {
                        error_output
                            .write_fmt(format_args!("Building: Configuration save failed: {:?}", e))
                            .expect("STDERR closed");
                        // Mark command as failed
                        ierr = 1;
                    }
                }
            }
        }
        None => match building.to_file() {
            Ok(()) => {
                if options.verbosity > 1 {
                    output
                        .write_all("Building: Configuration saved.\n".as_bytes())
                        .expect("STDOUT closed");
                }
            }
            Err(e) => {
                if options.verbosity > 0 {
                    error_output
                        .write_fmt(format_args!("Building: Configuration save failed: {:?}", e))
                        .expect("STDERR closed");
                }
                // Mark command as failed
                ierr = 1;
            }
        },
    }

    ierr
}

//==============================================================================
// Unit Tests

#[cfg(test)]
mod apartments_tests {

    use regex::Regex;
    use std::collections::HashMap;
    use std::fs;
    use std::io::{Error, ErrorKind};
    use std::path::Path;
    use std::path::PathBuf;

    use crate::apartments;
    use crate::apartments::{model, Apartment, Floor};
    use crate::app::{AppOptions, Commands};

    fn create_data_file(file: &Path) -> Result<(), Error> {
        let main_dir = model::try_find_main_directory(Some(2)).unwrap();

        let mut data_dir = PathBuf::from(main_dir.as_path());

        data_dir.push("data");
        data_dir.push(file);

        // Extend data file with data directory
        let data_file = PathBuf::from(data_dir.as_path());

        if let Some(p) = data_dir.parent() {
            data_dir = PathBuf::from(p);
        }

        let create_dir = match data_dir.try_exists() {
            Ok(exists) => match exists {
                true => false,
                false => true,
            },
            Err(_) => true,
        };

        if create_dir {
            match fs::create_dir_all(data_dir.as_path()) {
				Ok(()) => {
					println!("Data Directory '{}': Directory was created.", data_dir.display())
				},
		        Err(e) => {
		          return Err::<(), std::io::Error>(Error::new(
		               ErrorKind::Other,
		               format!(
		                   "Data Directory '{}' - Data File {:?}: Data Directory could not be created: {:?}",
		                   data_dir.display(),
		                   file.file_name(),
		                   e
		               )
		           ))
		        }
	    	}
        }

        // Create valid YAML file with empty Building
        fs::write(data_file, "---\nfloors: []\n".as_bytes())?;

        Ok(())
    }

    fn parse_debug_listing(listing: &str) -> Result<HashMap<u16, Floor>, Error> {
        let mut floors = HashMap::<u16, Floor>::new();

        let floors_exp = Regex::new(r"Floor No. (?P<floor_no>\d+):").map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!(
                    "Regex '{}': Expression Parsing failed: {:?}",
                    r"Floor No. (?P<floor_no>\d+):", e
                ),
            )
        })?;

        for cap in floors_exp.captures_iter(listing) {
            println!("flr cap dmp: {:?}", cap);

            let floor = Floor::from_string(&cap["floor_no"])?;

            floors.insert(floor.number, floor);
        }

        println!("Floors Res 1 dmp: {:?}", floors);

        let apts_exp = Regex::new("Apartment \\{ code: \"(?P<apt_code>[^\"]+)\", floor: \\d+, door: \"\\w+\", occupied: (?P<apt_occupied>true|false) \\}").
			map_err(|e| { Error::new(
				ErrorKind::Other,
				format!(
					"Regex '{}': Expression Parsing failed: {:?}",
					r"Floor No. (?P<floor_no>\d+):", e
				)
			)})?;

        for cap in apts_exp.captures_iter(listing) {
            println!("apt cap dmp: {:?}", cap);

            let apt_occupied = match &cap["apt_occupied"] {
                "true" => true,
                "false" => false,
                _ => false,
            };

            let apt = Apartment::from_code(&cap["apt_code"], apt_occupied);

            println!("apt (occ: '{:?}') dmp: {:?}", apt_occupied, apt);

            match floors.get_mut(&apt.floor) {
                Some(ref mut f) => f.apartments.push(apt),
                None => {}
            }
        }

        for floor in &mut floors {
            floor.1.apartments.sort();
        }

        Ok(floors)
    }

    /// List option for the default Building…
    /// Checking the listing debug output of the default Building through parsing it back with regex

    #[test]
    fn option_list() {
        //-------------------------------------
        // List Apartments

        let data_file = Path::new("apartments_list.yml");
        let command = Commands::Apartments {
            list: true,
            occupy: None,
            add: None,
        };
        let options = AppOptions {
            verbosity: 0,
            file: Some(data_file.to_string_lossy().to_string()),
            command: Some(command),
        };
        let mut output_raw = Vec::new();
        let mut error_raw = Vec::new();
        let expected_floors: Vec<u16> = vec![1, 2];
        let expected_apt_codes: Vec<Vec<&str>> =
            vec![vec!["1A", "1B", "1C"], vec!["2A", "2B", "2C"]];
        let expected_apt_doors: Vec<Vec<&str>> = vec![vec!["A", "B", "C"], vec!["A", "B", "C"]];

        assert!(create_data_file(data_file).is_ok());

        let ierr = apartments::main(&options, &mut output_raw, &mut error_raw);

        let output = String::from_utf8_lossy(&output_raw);
        let error = String::from_utf8_lossy(&error_raw);

        println!("Exit Code: '{}'", ierr);
        println!("STDOUT: '{}'", output);
        println!("STDERR: '{}'", error);

        assert_eq!(ierr, 0);

        let floors = parse_debug_listing(&output).unwrap();

        println!("Floors Res 2 dmp: {:?}", floors);

        assert_eq!(floors.len(), expected_floors.len());

        for flr_idx in 0..expected_floors.len() {
            assert!(floors.get(&expected_floors[flr_idx]).is_some());

            match floors.get(&expected_floors[flr_idx]) {
                Some(f) => {
                    for apt_idx in 0..expected_apt_codes[flr_idx].len() {
                        assert_eq!(
                            f.apartments[apt_idx].code,
                            expected_apt_codes[flr_idx][apt_idx]
                        );
                        assert_eq!(f.apartments[apt_idx].floor, expected_floors[flr_idx]);
                    }

                    for apt_idx in 0..expected_apt_doors[flr_idx].len() {
                        assert_eq!(
                            f.apartments[apt_idx].door,
                            expected_apt_doors[flr_idx][apt_idx]
                        );
                    }
                }
                None => {}
            }
        }
    }

    /// Occupy option for 1 Apartment and checking the listing debug output.
    /// Checking the listing debug output of the default Building through parsing it back with regex

    #[test]
    fn option_occupy() {
        //-------------------------------------
        // Occupy 3 apartments List Apartments
        // Checking the Debug Output of the default Building through back parsing it with Regex

        let data_file = Path::new("apartments_occupy.yml");
        let command = Commands::Apartments {
            list: true,
            occupy: Some("1c".to_string()),
            add: None,
        };
        let options = AppOptions {
            verbosity: 0,
            file: Some(data_file.to_string_lossy().to_string()),
            command: Some(command),
        };
        let mut output_raw = Vec::new();
        let mut error_raw = Vec::new();
        let expected_floors: Vec<u16> = vec![1, 2];
        let expected_apt_codes: Vec<Vec<&str>> =
            vec![vec!["1A", "1B", "1C"], vec!["2A", "2B", "2C"]];
        let expected_apt_doors: Vec<Vec<&str>> = vec![vec!["A", "B", "C"], vec!["A", "B", "C"]];
        let expected_apt_occupancies: Vec<Vec<bool>> =
            vec![vec![false, false, true], vec![false, false, false]];

        assert!(create_data_file(data_file).is_ok());

        let ierr = apartments::main(&options, &mut output_raw, &mut error_raw);

        let output = String::from_utf8_lossy(&output_raw);
        let error = String::from_utf8_lossy(&error_raw);

        println!("Exit Code: '{}'", ierr);
        println!("STDOUT: '{}'", output);
        println!("STDERR: '{}'", error);

        assert_eq!(ierr, 0);

        let floors = parse_debug_listing(&output).unwrap();

        println!("Floors Res 2 dmp: {:?}", floors);

        assert_eq!(floors.len(), expected_floors.len());

        for flr_idx in 0..expected_floors.len() {
            assert!(floors.get(&expected_floors[flr_idx]).is_some());

            match floors.get(&expected_floors[flr_idx]) {
                Some(f) => {
                    for apt_idx in 0..expected_apt_codes[flr_idx].len() {
                        assert_eq!(
                            f.apartments[apt_idx].code,
                            expected_apt_codes[flr_idx][apt_idx]
                        );
                        assert_eq!(f.apartments[apt_idx].floor, expected_floors[flr_idx]);
                    }

                    for apt_idx in 0..expected_apt_doors[flr_idx].len() {
                        assert_eq!(
                            f.apartments[apt_idx].door,
                            expected_apt_doors[flr_idx][apt_idx]
                        );
                    }

                    for apt_idx in 0..expected_apt_occupancies[flr_idx].len() {
                        assert_eq!(
                            f.apartments[apt_idx].occupied,
                            expected_apt_occupancies[flr_idx][apt_idx]
                        );
                    }
                }
                None => {}
            }
        }
    }

    /// Add option for 1 Apartment and checking the listing debug output.
    /// Checking the listing debug output of the default Building through parsing it back with regex

    #[test]
    fn option_add() {
        //-------------------------------------
        // Occupy 3 apartments List Apartments
        // Checking the Debug Output of the default Building through back parsing it with Regex

        let data_file = Path::new("apartments_add.yml");
        let command = Commands::Apartments {
            list: true,
            occupy: None,
            add: Some("5ac".to_string()),
        };
        let options = AppOptions {
            verbosity: 0,
            file: Some(data_file.to_string_lossy().to_string()),
            command: Some(command),
        };
        let mut output_raw = Vec::new();
        let mut error_raw = Vec::new();
        let expected_floors: Vec<u16> = vec![1, 2, 5];
        let expected_apt_codes: Vec<Vec<&str>> =
            vec![vec!["1A", "1B", "1C"], vec!["2A", "2B", "2C"], vec!["5AC"]];
        let expected_apt_doors: Vec<Vec<&str>> =
            vec![vec!["A", "B", "C"], vec!["A", "B", "C"], vec!["AC"]];
        let expected_apt_occupancies: Vec<Vec<bool>> = vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false],
        ];

        assert!(create_data_file(data_file).is_ok());

        let ierr = apartments::main(&options, &mut output_raw, &mut error_raw);

        let output = String::from_utf8_lossy(&output_raw);
        let error = String::from_utf8_lossy(&error_raw);

        println!("Exit Code: '{}'", ierr);
        println!("STDOUT: '{}'", output);
        println!("STDERR: '{}'", error);

        assert_eq!(ierr, 0);

        let floors = parse_debug_listing(&output).unwrap();

        println!("Floors Res 2 dmp: {:?}", floors);

        assert_eq!(floors.len(), expected_floors.len());

        for flr_idx in 0..expected_floors.len() {
            assert!(floors.get(&expected_floors[flr_idx]).is_some());

            match floors.get(&expected_floors[flr_idx]) {
                Some(f) => {
                    for apt_idx in 0..expected_apt_codes[flr_idx].len() {
                        assert_eq!(
                            f.apartments[apt_idx].code,
                            expected_apt_codes[flr_idx][apt_idx]
                        );
                        assert_eq!(f.apartments[apt_idx].floor, expected_floors[flr_idx]);
                    }

                    for apt_idx in 0..expected_apt_doors[flr_idx].len() {
                        assert_eq!(
                            f.apartments[apt_idx].door,
                            expected_apt_doors[flr_idx][apt_idx]
                        );
                    }

                    for apt_idx in 0..expected_apt_occupancies[flr_idx].len() {
                        assert_eq!(
                            f.apartments[apt_idx].occupied,
                            expected_apt_occupancies[flr_idx][apt_idx]
                        );
                    }
                }
                None => {}
            }
        }
    }
}
