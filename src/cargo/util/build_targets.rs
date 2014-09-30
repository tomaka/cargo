use core::PackageId;
use serialize::json;
use std::io::File;
use util::{Config, CargoResult, human};

/// Allows access to the build targets of the system.
pub struct BuildTargets<'a, 'b: 'a> {
    config: &'a Config<'b>
}

impl<'a, 'b> BuildTargets<'a, 'b> {
    /// Builds an access to the build targets.
    pub fn new(config: &'a Config<'b>) -> BuildTargets<'a, 'b> {
        BuildTargets {
            config: config,
        }
    }

    /// Installs a new target.
    pub fn install(&self, pkg: &PackageId) -> CargoResult<()> {
        let mut content = try!(self.get_file_content());
        let mut found = false;

        for mut element in content.iter_mut() {
            if element.name.as_slice() == pkg.get_name() {
                element.package_id = pkg.clone();
                found = true;
                break;
            }
        }

        if !found {
            content.push(Target {
                name: pkg.get_name().to_string(),
                package_id: pkg.clone(),
            });
        }

        self.set_file_content(content)
    }
    
    /// Returns the path of the targets list.
    fn get_build_targets_file(&self) -> Path {
        self.config.build_targets_path().join("targets.json")
    }

    /// Returns the content of the targets list file.
    fn get_file_content(&self) -> CargoResult<Vec<Target>> {
        use serialize::Decodable;
        use std::io;

        let path = self.get_build_targets_file();

        let mut file = match File::open(&path) {
            Err(ref e) if e.kind == io::FileNotFound => return Ok(Vec::new()),
            Err(ref e) => return Err(human(format!("Unable to open {}: {}", path.display(), e))),
            Ok(file) => file
        };

        let mut decoder = match json::from_reader(&mut file) {
            Err(e) => return Err(human(format!("Unable to parse {}: {}", path.display(), e))),
            Ok(json) => json::Decoder::new(json)
        };

        Decodable::decode(&mut decoder)
            .map_err(|e| human(format!("Build targets file is corrupt: {}", e)))
    }

    /// Modifies the content of the targets list file.
    fn set_file_content(&self, content: Vec<Target>) -> CargoResult<()> {
        use serialize::Encodable;
        use std::io;
        use std::io::fs::PathExtensions;

        let path = self.get_build_targets_file();

        if !path.dir_path().exists() {
            use std::io::fs::mkdir;
            try!(mkdir(&path.dir_path(), io::FilePermission::all())
                .map_err(|e| human(format!("Could not create the path to {}: {}",
                path.display(), e))))
        }

        let mut file = match File::open_mode(&path, io::Truncate, io::Write) {
            Err(ref e) => return Err(human(format!("Unable to open {}: {}", path.display(), e))),
            Ok(file) => file
        };

        let mut encoder = json::Encoder::new(&mut file);
        content.encode(&mut encoder)
            .map_err(|e| human(format!("Unable to write on build targets file: {}", e)))
    }
}

#[deriving(Decodable, Encodable)]
struct Target {
    name: String,
    package_id: PackageId,
}
