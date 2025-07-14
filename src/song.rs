pub struct Song {
    pub file_path : String,
    //pub source : Decoder<BufReader<File>>,
}

impl Song {
    pub fn new(file_path : &str) -> Self {
        //let buffered = BufReader::new(file);
        Self {
            file_path : file_path.to_string(),
            //source : Decoder::try_from(buffered).unwrap(),
        }
    }
}