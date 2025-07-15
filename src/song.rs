pub struct Song {
    pub file_path : String,
    pub title : String,
    //pub source : Decoder<BufReader<File>>,
}

impl Song {
    pub fn new(file_path : &str) -> Self {
        //let buffered = BufReader::new(file);
        let path = std::path::Path::new(&file_path);
        

        Self {
            file_path : file_path.to_string(),
            //source : Decoder::try_from(buffered).unwrap(),
            title : path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),

        }
    }
}