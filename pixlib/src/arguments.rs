pub struct Arguments {
    pub path_to_iso: String,
    pub path_to_file: String,
    pub output_path: Option<String>,
}

pub fn get_arguments() -> Result<Arguments, ()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        return Err(());
    }
    let path_to_iso = args[1].clone();
    let path_to_file = args[2].to_ascii_uppercase().replace('\\', "/");
    let output_path = args.get(3).map(|s| s.to_owned());

    Ok(Arguments {
        path_to_iso,
        path_to_file,
        output_path,
    })
}
