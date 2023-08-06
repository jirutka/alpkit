use std::env;
use std::process::exit;

use schemars::gen::SchemaSettings;

use alpkit::apkbuild::Apkbuild;
use alpkit::package::Package;

fn main() {
    let gen = SchemaSettings::draft2019_09().into_generator();

    let schema = match env::args().nth(1).as_deref() {
        Some("apk") => gen.into_root_schema_for::<Package>(),
        Some("apkbuild") => gen.into_root_schema_for::<Apkbuild>(),
        _ => {
            eprintln!("Usage: schema-gen (apk | apkbuild)");
            exit(1);
        }
    };

    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
