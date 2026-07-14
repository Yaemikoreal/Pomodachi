const SERVICE: &str = "com.pomodachi.app";
const USERNAME: &str = "api_key";

pub fn store_api_key(key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, USERNAME).map_err(|e| e.to_string())?;
    entry.set_password(key).map_err(|e| e.to_string())
}

pub fn get_api_key() -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(SERVICE, USERNAME).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn delete_api_key() -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, USERNAME).map_err(|e| e.to_string())?;
    entry.delete_credential().map_err(|e| e.to_string())
}
