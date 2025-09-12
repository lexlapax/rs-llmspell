use sled::Db;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = sled::open("rs-llmspell.db")?;
    
    println!("Keys in database:");
    for item in db.iter() {
        let (key, _value) = item?;
        let key_str = String::from_utf8_lossy(&key);
        println!("  {}", key_str);
    }
    
    Ok(())
}