use std::env;
use std::error;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufReader, Write as _};
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;

use alpkit::apkbuild::ApkbuildReader;
use alpkit::package::Package;

use argp::FromArgs;

const PROG_NAME: &str = env!("CARGO_PKG_NAME");
const PROG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Extract metadata from Alpine's APK packages and APKBUILDs.
#[derive(Debug, FromArgs)]
#[argp(footer = "Please report issues at <https://github.com/jirutka/alpkit>.")]
struct AppOpts {
    /// Format the output to be human-readable.
    #[argp(switch, short = 'p', global)]
    pretty_print: bool,

    /// Show program name and version.
    #[argp(switch, short = 'V')]
    version: bool,

    #[argp(subcommand)]
    action: Option<Action>,
}

/// Read APKv2 package.
#[derive(Debug, FromArgs)]
#[argp(subcommand, name = "apk")]
struct ApkOpts {
    /// Don't read files (data) section.
    #[argp(switch)]
    no_files: bool,

    /// Path to an APK package.
    #[argp(positional, arg_name = "file")]
    file: PathBuf,
}

/// Read APKBUILD file.
#[derive(Debug, FromArgs)]
#[argp(subcommand, name = "apkbuild")]
struct ApkbuildOpts {
    /// Set given variable(s) in the environment for the APKBUILD evaluation.
    #[argp(
        option,
        short = 'e',
        arg_name = "VAR=VALUE",
        from_str_fn(parse_env_var)
    )]
    env: Vec<(OsString, OsString)>,

    /// Do not clear environment variables before evaluating APKBUILD.
    /// By default, only variables specified by --env are set.
    #[argp(switch, short = 'k')]
    keep_env: bool,

    /// Use <shell> to evaluate APKBUILD (default is /bin/sh).
    #[argp(
        option,
        short = 's',
        arg_name = "shell",
        default = "OsString::from(\"/bin/sh\")"
    )]
    shell: OsString,

    /// If shell evaluation of APKBUILD exceeds <msec> milliseconds, kill it.
    /// Default is 250, use 0 to disable.
    #[argp(option, short = 'T', arg_name = "msec", default = "250")]
    timeout: u64,

    /// Path to an APKBUILD file.
    #[argp(positional, arg_name = "apkbuild")]
    file: PathBuf,
}

#[derive(Debug, FromArgs)]
#[argp(subcommand)]
enum Action {
    Apk(ApkOpts),
    Apkbuild(ApkbuildOpts),
}

fn main() {
    let args: AppOpts = argp::from_env();

    if args.version {
        println!("{} {}", PROG_NAME, PROG_VERSION);
        exit(0);
    }

    if let Err(e) = run(args) {
        eprintln!("{}", format_error_message(e));
        exit(1);
    }
}

fn run(args: AppOpts) -> Result<(), Box<dyn std::error::Error>> {
    let action = args.action.ok_or("no subcommand specified")?;

    match action {
        Action::Apk(opts) => {
            let reader = File::open(&opts.file).map(BufReader::new).map_err(|e| {
                format!("cannot open file '{}': {}", &opts.file.to_string_lossy(), e)
            })?;

            if !opts.file.is_file() {
                return Err(
                    format!("'{}' is not a regular file", &opts.file.to_string_lossy()).into(),
                );
            }

            let pkg = if opts.no_files {
                Package::load_without_files(reader)?
            } else {
                Package::load(reader)?
            };

            dump_json(&pkg, args.pretty_print)?;
        }
        Action::Apkbuild(opts) => {
            let apkbuild = ApkbuildReader::new()
                .envs(opts.env)
                .inherit_env(opts.keep_env)
                .shell_cmd(opts.shell)
                .time_limit(Duration::from_millis(opts.timeout))
                .read_apkbuild(&opts.file)?;

            dump_json(&apkbuild, args.pretty_print)?;
        }
    };

    Ok(())
}

fn parse_env_var(s: &str) -> Result<(OsString, OsString), String> {
    s.split_once('=')
        .map(|(k, v)| (k.into(), v.into()))
        .ok_or_else(|| format!("expected VAR=VALUE, but got: '{}'", s))
}

fn dump_json<T: ?Sized + serde::Serialize>(
    value: &T,
    pretty: bool,
) -> Result<(), serde_json::Error> {
    if pretty {
        serde_json::to_writer(io::stdout(), value)
    } else {
        serde_json::to_writer_pretty(io::stdout(), value)?;
        let _ = io::stdout().write(b"\n");
        Ok(())
    }
}

fn format_error_message(error: Box<dyn error::Error>) -> String {
    let mut msg = String::from(PROG_NAME);

    let mut source = Some(error.as_ref());
    while let Some(e) = source {
        msg.push_str(": ");
        msg.push_str(&e.to_string());

        source = e.source();
    }
    msg
}
