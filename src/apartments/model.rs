use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::{Component, Path, PathBuf};

extern crate serde;
extern crate serde_yaml;

use serde_derive::{Deserialize, Serialize};

const BUILDING_FILE: &'static str = "building.yaml";

//==============================================================================
// Structure Apartment Declaration

/// Structure representing an apartment
#[derive(Debug, Default, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct Apartment {
    pub code: String,
    pub floor: u16,
    pub door: String,
    pub occupied: bool,
}

//==============================================================================
// Structure Floor Declaration

/// Structure representing a building floor containing several apartments
#[derive(Debug, Default, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct Floor {
    pub number: u16,
    pub apartments: Vec<Apartment>,
}

//==============================================================================
// Structure Building Declaration

/// Structure representing an apartment building containing several floors
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Building {
    pub floors: Vec<Floor>,

    #[serde(skip_serializing)]
    pub verbosity: Option<u8>,
}

//==============================================================================
// Structure Apartment Implementation

impl Apartment {
    pub fn from_code(apartment_code: &str, occupied: bool) -> Self {
        let mut floor_number = String::new();
        let mut door_letter = String::new();
        let apt_upper = apartment_code.to_uppercase();

        for apt_char in apt_upper.chars() {
            if apt_char.is_numeric() {
                floor_number.push(apt_char);
            } else if apt_char.is_alphabetic() {
                door_letter.push(apt_char);
            }
        }

        let floor = match floor_number.parse::<u16>() {
            Ok(u) => u,
            Err(e) => {
                eprintln!("failed to parse '{}': {:?}", floor_number, e);
                0
            }
        };

        Apartment {
            code: apt_upper,
            floor: floor,
            door: door_letter,
            occupied: occupied,
        }
    }

    pub fn from_floor_door(floor: u16, door: &str, occupied: bool) -> Self {
        let door_letter = door.to_uppercase();
        Apartment {
            code: format!("{}{}", floor, door_letter.as_str()),
            floor: floor,
            door: door_letter,
            occupied: occupied,
        }
    }
}

//==============================================================================
// Structure Floor Implementation

impl Floor {
    pub fn from_string(floor_number: &str) -> Result<Self, Error> {
        match floor_number.parse::<u16>() {
            Ok(u) => Ok(Floor {
                number: u,
                apartments: Vec::<Apartment>::new(),
            }),
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("Building: Conversion to YAML string failed: {:?}", e),
            )),
        }
    }
}

//==============================================================================
// Structure Building Implementation

impl Building {
    pub fn from_file(verbosity: Option<u8>) -> Self {
        Self::from_custom_file(Path::new(&*BUILDING_FILE), verbosity)
    }

    pub fn from_custom_file(file: &Path, verbosity: Option<u8>) -> Self {
        let mut building: Option<Building> = None;
        let mut data_file: Option<PathBuf> = None;
        let verbose = match verbosity {
            Some(v) => v,
            None => 1,
        };

        let main_dir: Option<PathBuf> = match try_find_main_directory(verbosity) {
            Ok(d) => Some(d),
            Err(e) => {
                eprintln!("Main Directory: Directory could not be found: {:?}", e);
                None
            }
        };

        if let Some(d) = main_dir {
            if verbose > 1 {
                println!("Main Directory: '{}'", d.display());
            }

            let mut data_dir = PathBuf::from(d.as_path());

            data_dir.push("data");

            data_file = match try_find_file(data_dir.as_path(), file, verbosity) {
                Ok(f) => Some(f),
                Err(_) => match try_find_file(d.as_path(), file, verbosity) {
                    Ok(f) => Some(f),
                    Err(e) => {
                        eprintln!(
                            "Data File '{}': File could not be found: {:?}",
                            &file.display(),
                            e
                        );
                        None
                    }
                },
            };
        }

        if let Some(f) = data_file {
            building = match try_building_from_file(&f, verbosity) {
                Ok(cfg) => Some(cfg),
                Err(e) => {
                    eprintln!("Data File {:?}: File could not be read: {:?}", f, e);
                    None
                }
            };
        }

        match building {
            Some(b) => b,
            None => {
                eprintln!("Falling back to default configuration ...");
                Building::default()
            }
        }
    }

    pub fn to_file(&self) -> Result<(), Error> {
        self.to_custom_file(Path::new(&*BUILDING_FILE))
    }

    pub fn to_yaml(&self) -> Result<String, Error> {
        // Serialize it to a YAML string.
        let yaml = serde_yaml::to_string(self).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Building: Conversion to YAML string failed: {:?}", e),
            )
        })?;

        Ok(yaml)
    }

    pub fn print(&self, output: &mut impl io::Write) -> Result<(), Error> {
        let yaml = self.to_yaml()?;

        output.write_fmt(format_args!("{}", yaml))?;

        Ok(())
    }

    pub fn to_custom_file(&self, file: &Path) -> Result<(), Error> {
        let mut data_file = PathBuf::from(file);
        let verbose = match self.verbosity {
            Some(v) => v,
            None => 1,
        };

        if !path_is_absolute(data_file.as_path()) {
            let main_dir: Option<PathBuf> = match try_find_main_directory(self.verbosity) {
                Ok(d) => Some(d),
                Err(e) => {
                    eprintln!("Main Directory: Directory could not be found: {:?}", e);
                    None
                }
            };

            if let Some(d) = main_dir {
                if verbose > 1 {
                    println!("Main Directory: '{}'", d.display());
                }

                let mut data_dir = PathBuf::from(d.as_path());

                data_dir.push("data");
                data_dir.push(data_file);

                // Extend data file with data directory
                data_file = PathBuf::from(data_dir.as_path());

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
							if verbose > 0 {
			                  	println!("Data Directory '{}': Directory was created.", data_dir.display())
							}
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
            } else {
                //Config File does not exist
                return Err::<(), std::io::Error>(Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "Main Directory - Data File {:?}: Main Directory cannot be found!",
                        file.file_name()
                    ),
                ));
            }
        }

        // Serialise Building data into the file
        try_building_to_file(&self, data_file.as_path())
    }
}

//==============================================================================
// Auxiliary Functions

fn try_find_file(current: &Path, file: &Path, verbosity: Option<u8>) -> Result<PathBuf, Error> {
    let mut search_dir: Option<&Path> = Some(current);
    let mut find_file: Option<PathBuf> = None;
    let verbose = match verbosity {
        Some(v) => v,
        None => 1,
    };

    while search_dir.is_some() && find_file.is_none() {
        if let Some(d) = search_dir {
            if verbose > 1 {
                println!("Search Directory: '{}'", d.display());
            }

            let mut search_file = PathBuf::from(d);

            search_file.push(file);

            match search_file.try_exists() {
                Ok(exists) => {
                    find_file = match exists {
                        true => Some(search_file),
                        false => {
                            search_dir = d.parent();
                            None
                        }
                    }
                }
                // Continue searching in Parent Directory
                Err(_) => search_dir = d.parent(),
            }
        } //if let Some(d) = search_dir
    } //while search_dir.is_some() && find_file.is_none()

    if let Some(f) = find_file {
        Ok(f)
    } else {
        //Config File does not exist
        Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "Working Directory '{}' - Data File {:?}: file does not exist in any parent directory!",
                current.display(),
                file.file_name()
            ),
        ))
    } //if let Some(f) = find_file
}

pub fn try_find_main_directory(verbosity: Option<u8>) -> Result<PathBuf, Error> {
    let cargo_file = Path::new("Cargo.toml");
    let mut find_dir: Option<PathBuf> = None;
    let verbose = match verbosity {
        Some(v) => v,
        None => 1,
    };

    let mut search_dir = std::env::current_dir().map_err(|e| {
        Error::new(
            ErrorKind::NotFound,
            format!(
                "Working Directory: find directory failed with Error: {:?}",
                e
            ),
        )
    })?;
    if verbose > 1 {
        println!("Working Directory: '{}'", search_dir.display());
    }

    match try_find_file(search_dir.as_path(), cargo_file, verbosity) {
        Ok(f) => {
            find_dir = match f.parent() {
                Some(p) => Some(p.to_path_buf()),
                None => Some(f.to_path_buf()),
            }
        }
        Err(_) => {}
    }

    if find_dir.is_none() {
        let module_path = std::env::current_exe().map_err(|e| {
            Error::new(
                ErrorKind::NotFound,
                format!(
                    "Module Path: find executable path failed with Error: {:?}",
                    e
                ),
            )
        })?;
        search_dir = fs::canonicalize(module_path)?;

        if let Some(d) = search_dir.parent() {
            match try_find_file(d, cargo_file, verbosity) {
                Ok(f) => {
                    find_dir = match f.parent() {
                        Some(p) => Some(p.to_path_buf()),
                        None => Some(f.to_path_buf()),
                    }
                }
                Err(_) => {}
            }
        }
    }

    if let Some(f) = find_dir {
        Ok(f)
    } else {
        //Config File does not exist
        Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "Directory '{}' - Data File {:?}: file does not exist in any parent directory!",
                search_dir.display(),
                cargo_file.file_name()
            ),
        ))
    } //if let Some(f) = find_dir
}

fn try_building_from_file(file: &Path, verbosity: Option<u8>) -> Result<Building, Error> {
    let building_yaml = fs::read_to_string(file).map_err(|e| {
        Error::new(
            ErrorKind::NotFound,
            format!(
                "Data File {:?}: read file failed with Error: '{:?}'",
                file, e
            ),
        )
    })?;
    let mut building: Building = serde_yaml::from_str(&building_yaml).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!(
                "Data File {:?}: parse file failed with Error: '{:?}'",
                file.file_name(),
                e
            ),
        )
    })?;

    building.verbosity = verbosity;

    Ok(building)
}

fn try_building_to_file(building: &Building, file: &Path) -> Result<(), Error> {
    // Serialize it to a YAML string.
    let yaml = building.to_yaml()?;

    fs::write(file, yaml.as_bytes())?;

    Ok(())
}

pub fn path_is_absolute(file: &Path) -> bool {
    let mut components = file.components();

    components.next() == Some(Component::RootDir)
}

#[allow(dead_code)]
pub fn find_path_parent(current: &Path, name: &str) -> Option<PathBuf> {
    let mut odir = None;

    let osearch = Some(OsStr::new(name));

    for p in current.ancestors() {
        if odir.is_none() && p.is_dir() && p.file_name() == osearch {
            odir = Some(p);
        }
    }

    if let Some(d) = odir {
        odir = d.parent();
    }

    match odir {
        Some(d) => Some(PathBuf::from(d)),
        None => None,
    }
}

//==============================================================================
// Unit Tests

#[test]
fn apartment_from_code() {
    //-------------------------------------
    // Create Apartment Structure from Apartment Code

    let apt_codes = vec!["7c", "11A", "%13AB", "#17C!"];
    let apt_occupancies = vec![false, true, true, false];
    let mut apts = Vec::<Apartment>::with_capacity(4);
    let expected_floors: Vec<u16> = vec![7, 11, 13, 17];
    let expected_doors = vec!["C", "A", "AB", "C"];
    let expected_occupancies = vec![false, true, true, false];

    for apt_idx in 0..apt_codes.len() {
        apts.push(Apartment::from_code(
            apt_codes[apt_idx],
            apt_occupancies[apt_idx],
        ));
    }

    println!("apts: {:?}", apts);

    assert_eq!(apts.len(), apt_codes.len());

    for apt_idx in 0..apt_codes.len() {
        assert_eq!(apts[apt_idx].code, apt_codes[apt_idx].to_uppercase());
        assert_eq!(apts[apt_idx].floor, expected_floors[apt_idx]);
        assert_eq!(apts[apt_idx].door, expected_doors[apt_idx]);
        assert_eq!(apts[apt_idx].occupied, expected_occupancies[apt_idx]);
    }
}

#[test]
fn apartment_from_floor_door() {
    //-------------------------------------
    // Create Apartment from Floor and Door

    let apt_floors = vec![7, 11, 13, 17];
    let apt_doors = vec!["c", "E", "AB", "G"];
    let apt_occupancies = vec![false, true, true, false];
    let mut apts = Vec::<Apartment>::with_capacity(4);
    let expected_codes = vec!["7C", "11E", "13AB", "17G"];
    let expected_floors: Vec<u16> = vec![7, 11, 13, 17];
    let expected_doors = vec!["C", "E", "AB", "G"];
    let expected_occupancies = vec![false, true, true, false];

    for apt_idx in 0..apt_floors.len() {
        apts.push(Apartment::from_floor_door(
            apt_floors[apt_idx],
            apt_doors[apt_idx],
            apt_occupancies[apt_idx],
        ));
    }

    println!("apts: {:?}", apts);

    assert_eq!(apts.len(), expected_codes.len());

    for apt_idx in 0..expected_codes.len() {
        assert_eq!(apts[apt_idx].code, expected_codes[apt_idx]);
        assert_eq!(apts[apt_idx].floor, expected_floors[apt_idx]);
        assert_eq!(apts[apt_idx].door, expected_doors[apt_idx]);
        assert_eq!(apts[apt_idx].occupied, expected_occupancies[apt_idx]);
    }
}
