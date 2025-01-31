use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

//use text_diff::print_diff;

pub fn cache_leaderboard(id: i32, text: String) -> bool {
    let path_str = format!("./cache/{}.cache", id.to_string());
    let path = Path::new(&path_str);
    if let Err(_e) = File::open(path) {
        // Cache does not exist, create it.
        let mut ofp = File::create(path).expect("File opening error for editing the cache file");
        ofp.write_all(text.as_bytes())
            .expect("Cache Writting Error");
        return true;
    }

    // Check cache
    let ifp = File::open(path).expect("Error opening cache files");
    let mut buf_reader = BufReader::new(ifp);
    let mut cache_contents = String::new();
    buf_reader
        .read_to_string(&mut cache_contents)
        .expect("Error reading the buffer");
    // This removes the "totalLeaderboardEntries" value. This makes it so we don't need to do as many cache re-writes, as we only care about updates past a certain point.
    let split = text.split("totalLeaderboardEntries").collect::<Vec<&str>>();
    // Reformat the string so that we can compare properly.
    let format_text = format!("{}-{}", split[0], split[2]);
    //print_diff(&cache_contents, &text,"<");
    if format_text.eq(&cache_contents) {
        //println!("Content not updated for map {}", id);
        false
    } else {
        let mut ofp = File::create(path).expect("Error creating file to write to for cache");
        ofp.write_all(format_text.as_bytes())
            .expect("Error writing to cache files");
        true
    }
}
