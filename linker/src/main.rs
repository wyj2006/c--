use anyhow::Result;
use clap::{Parser, ValueHint};
use indexmap::IndexMap;
use linker::{
    Linker,
    archive::{ARCHIVE_MAGIC, Archive},
};
use object::File;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser)]
pub struct Cli {
    pub inputs: Vec<String>,
    #[arg(short, long)]
    pub output: Option<String>,
    #[arg(short = 'L', long = "lib-path", value_hint = ValueHint::DirPath)]
    pub lib_paths: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let mut cli = Cli::parse();
    cli.lib_paths.insert(0, Path::new("").to_path_buf());

    let mut paths = vec![];
    for path in &cli.inputs {
        for parent_path in &cli.lib_paths {
            paths.push(parent_path.join(path));
        }
    }

    let mut datas = IndexMap::new();
    for path in &paths {
        datas.insert(
            path.file_name().unwrap().to_str().unwrap(),
            match fs::read(path) {
                Ok(t) => t,
                Err(_) => continue,
            },
        );
    }

    let mut archives = vec![];

    let mut datas = {
        let mut t = IndexMap::new();
        for (path, data) in &datas {
            if data.starts_with(ARCHIVE_MAGIC) {
                archives.push(Archive::parse(data)?);
            } else {
                t.insert(*path, &**data);
            }
        }
        t
    };

    for archive in &archives {
        for member in &archive.members {
            datas.insert(member.name.as_str(), member.data.as_slice());
        }
    }

    let mut linker = Linker::new(
        {
            let mut inputs = IndexMap::new();
            for (path, data) in datas {
                inputs.insert(
                    path,
                    match File::parse(data) {
                        Ok(t) => t,
                        Err(_) => continue,
                    },
                );
            }
            inputs
        },
        if let Some(t) = cli.output {
            t
        } else {
            "a.exe".to_string()
        },
    );

    linker.link()?;

    Ok(())
}
