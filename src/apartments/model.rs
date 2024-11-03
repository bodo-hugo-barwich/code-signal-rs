use std::fs;
//use std::fs::File;
use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::path::{Component, Path, PathBuf};

extern crate serde;
extern crate serde_yaml;

use serde_derive::{Deserialize, Serialize};

const BUILDING_FILE: &'static str = "building.yaml";

//==============================================================================
// Structure Apartment Declaration

/// Structure representing an apartment
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Apartment {
    pub code: String,
    pub floor: u16,
    pub door: String,
    pub occupied: bool,
}

//==============================================================================
// Structure Floor Declaration

/// Structure representing a building floor containing several apartments
#[derive(Debug, Default, Serialize, Deserialize)]
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
// Structure Apartment Implementation

impl Building {
    pub fn from_file() -> Self {
        Self::from_custom_file(Path::new(&*BUILDING_FILE))
    }

    pub fn from_custom_file(file: &Path) -> Self {
        let mut building: Option<Building> = None;
        let mut data_file: Option<PathBuf> = None;

        let main_dir: Option<PathBuf> = match try_find_main_directory() {
            Ok(d) => Some(d),
            Err(e) => {
                eprintln!("Main Directory: Directory could not be found: {:?}", e);
                None
            }
        };

        if let Some(d) = main_dir {
            println!("Main Directory: '{}'", d.display());

            let mut data_dir = PathBuf::from(d.as_path());

            data_dir.push("data");

            data_file = match try_find_file(data_dir.as_path(), file) {
                Ok(f) => Some(f),
                Err(_) => match try_find_file(d.as_path(), file) {
                    Ok(f) => Some(f),
                    Err(e) => {
                        eprintln!(
                            "Data File '{}': File could not be found: {:?}",
                            BUILDING_FILE, e
                        );
                        None
                    }
                },
            };
        }

        if let Some(f) = data_file {
            building = match try_building_from_file(&f) {
                Ok(cfg) => Some(cfg),
                Err(e) => {
                    eprintln!("Data File {:?}: File could not be read: {:?}", f, e);
                    None
                }
            };
        }

        if building.is_none() {
            eprintln!("Falling back to default configuration ...");
            building = Some(Building::default());
        }

        match building {
            Some(b) => b,
            None => Building::default(),
        }
    }

    pub fn to_file(&self) -> Result<(), Error> {
        self.to_custom_file(Path::new(&*BUILDING_FILE))
    }

    pub fn to_custom_file(&self, file: &Path) -> Result<(), Error> {
        let mut data_file = PathBuf::from(file);

        if !path_is_absolute(data_file.as_path()) {
            let main_dir: Option<PathBuf> = match try_find_main_directory() {
                Ok(d) => Some(d),
                Err(e) => {
                    eprintln!("Main Directory: Directory could not be found: {:?}", e);
                    None
                }
            };

            if let Some(d) = main_dir {
                println!("Main Directory: '{}'", d.display());

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

fn try_find_file(current: &Path, file: &Path) -> Result<PathBuf, Error> {
    let mut search_dir: Option<&Path> = Some(current);
    let mut find_file: Option<PathBuf> = None;

    while search_dir.is_some() && find_file.is_none() {
        if let Some(d) = search_dir {
            println!("Search Directory: '{}'", d.display());

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

fn try_find_main_directory() -> Result<PathBuf, Error> {
    let cargo_file = Path::new("Cargo.toml");
    let mut find_dir: Option<PathBuf> = None;

    let mut search_dir = std::env::current_dir().map_err(|e| {
        Error::new(
            ErrorKind::NotFound,
            format!(
                "Working Directory: find directory failed with Error: {:?}",
                e
            ),
        )
    })?;
    println!("Working Directory: '{}'", search_dir.display());

    match try_find_file(search_dir.as_path(), cargo_file) {
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
            match try_find_file(d, cargo_file) {
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

fn try_building_from_file(file: &Path) -> Result<Building, Error> {
    let building_yaml = fs::read_to_string(file).map_err(|e| {
        Error::new(
            ErrorKind::NotFound,
            format!(
                "Data File {:?}: read file failed with Error: '{:?}'",
                file, e
            ),
        )
    })?;
    let building: Building = serde_yaml::from_str(&building_yaml).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!(
                "Data File {:?}: parse file failed with Error: '{:?}'",
                file.file_name(),
                e
            ),
        )
    })?;

    Ok(building)
}

fn try_building_to_file(building: &Building, file: &Path) -> Result<(), Error> {
    // Serialize it to a YAML string.
    let yaml = serde_yaml::to_string(building).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Building: Conversion to YAML string failed: {:?}", e),
        )
    })?;

    fs::write(file, yaml.as_bytes())?;

    Ok(())
}

fn path_is_absolute(file: &Path) -> bool {
    let mut components = file.components();

    components.next() == Some(Component::RootDir)
}

#[allow(dead_code)]
fn find_path_parent(current: &Path, name: &str) -> Option<PathBuf> {
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
