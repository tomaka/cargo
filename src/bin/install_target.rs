use std::os;
use docopt;

use cargo::core::MultiShell;
use cargo::core::dependency::Dependency;
use cargo::core::resolver;
use cargo::core::source::{Source, SourceId};
use cargo::util::build_targets::BuildTargets;
use cargo::util::{Config, CliResult, CliError};

docopt!(Options, "
Installs a target to be used later while compiling.

Usage:
    cargo install-target [options] PACKAGE URL

Options:
    -h, --help               Print this message
    -v, --verbose            Use verbose output
")

pub fn execute(options: Options, shell: &mut MultiShell) -> CliResult<Option<()>> {
    shell.set_verbose(options.flag_verbose);
    debug!("executing; cmd=cargo-install-target; args={}", os::args());

    let mut config = try!(Config::new(shell, None, None)
        .map_err(|e| CliError::from_boxed(e, 1)));

    // resolving the required package
    let package_id = {
        let source_id = SourceId::from_url(options.arg_URL);

        let mut source = source_id.load(&mut config);
        try!(source.update().map_err(|e| CliError::from_boxed(e, 1)));

        let summary = try!(source.query(&Dependency::new_override(options.arg_PACKAGE.as_slice(),
            &source_id)).map_err(|e| CliError::from_boxed(e, 1)));
        let summary = try!(summary.into_iter().next().ok_or(
            CliError::new(format!("Could not find package {}", options.arg_PACKAGE),
            1)));

        let package_id = summary.get_package_id();

        try!(source.download([package_id.clone()].as_slice())
            .map_err(|e| CliError::from_boxed(e, 1)));

        let package = try!(source.get([package_id.clone()].as_slice())
            .map_err(|e| CliError::from_boxed(e, 1)));
        let package = try!(package.into_iter().next().ok_or(
            CliError::new(format!("Could not find package {}", options.arg_PACKAGE),
            1)));

        if package.get_manifest().get_build_target().is_none() {
            let msg = "The target package doesn't contain a `target` element in its manifest";
            return Err(CliError::new(msg, 1));
        }

        package_id.clone()
    };

    try!(BuildTargets::new(&config).install(&package_id)
        .map_err(|e| CliError::from_boxed(e, 1)));

    Ok(None)
}
