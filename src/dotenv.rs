use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use eyre::{Context, OptionExt, eyre};

pub fn fetch_from_file(
    path: impl AsRef<Path>,
) -> eyre::Result<HashMap<String, String, ahash::RandomState>> {
    let mut output = HashMap::default();

    let f = File::open(path).wrap_err("unable to open the specified file")?;
    let reader = BufReader::new(f);
    let mut lines: std::io::Lines<BufReader<File>> = reader.lines();

    loop {
        match lines.next() {
            Some(Ok(line)) => {
                // technically we are hitting an edge-case when the required env variable
                // if its name has a `=` (like `"KE=EY"="VALUE"`) in it, but iirc
                // there are no real cases for this
                let (name_raw, value_raw) = line
                    .split_once('=')
                    .ok_or_eyre("got unparseable line in dotenv file")?;

                // try to add and check if it's a duplicate
                if output
                    .insert(
                        extract_value(name_raw).to_owned(),
                        extract_value(value_raw).to_owned(),
                    )
                    .is_some()
                {
                    return Err(eyre!("tried to parse a duplicate value '{name_raw}'"));
                }
            }

            Some(Err(e)) => {
                return Err(eyre!("unable to continue reading the file: {e}"));
            }

            None => {
                // EOF

                break;
            }
        }
    }

    if output.is_empty() {
        return Err(eyre!("buffer remains empty after parsing the env file?"));
    }

    Ok(output)
}

/// Extract `FOO` from `"FOO"`
fn extract_value(input: &str) -> &str {
    input.trim_matches('"')
}
