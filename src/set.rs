use crate::storage_result::{StorageError, StorageResult};

#[derive(Debug, PartialEq, Eq)]
pub enum KeyExistence {
    NX,
    XX,
}
#[derive(Debug, PartialEq, Eq)]
pub enum KeyExpiry {
    EX(u64),
    PX(u64),
}
#[derive(Debug, PartialEq, Eq)]
pub struct SetArgs {
    pub expiry: Option<KeyExpiry>,
    pub existence: Option<KeyExistence>,
    pub get: bool,
}
impl SetArgs {
    pub fn new() -> Self {
        SetArgs {
            expiry: None,
            existence: None,
            get: false,
        }
    }
}
pub fn parse_set_arguments(agruments: &Vec<String>) -> StorageResult<SetArgs> {
    let mut args = SetArgs::new();
    let mut idx: usize = 0;
    loop {
        if idx >= agruments.len() {
            break;
        }
        match agruments[idx].to_lowercase().as_str() {
            "nx" => {
                if args.existence == Some(KeyExistence::XX) {
                    return Err(StorageError::CommandSyntaxError(agruments.join(" ")));
                }
                args.existence = Some(KeyExistence::NX);
                idx += 1;
            }
            "xx" => {
                if args.existence == Some(KeyExistence::NX) {
                    return Err(StorageError::CommandSyntaxError(agruments.join(" ")));
                }
                args.existence = Some(KeyExistence::XX);
                idx += 1;
            }
            "ex" => {
                if let Some(KeyExpiry::PX(_)) = args.expiry {
                    return Err(StorageError::CommandSyntaxError(agruments.join(" ")));
                }
                if idx + 1 >= agruments.len() {
                    return Err(StorageError::CommandSyntaxError(agruments.join(" ")));
                }
                let value: u64 = agruments[idx + 1]
                    .parse::<u64>()
                    .map_err(|_| StorageError::CommandSyntaxError(agruments.join(" ")))?;

                args.expiry = Some(KeyExpiry::EX(value));
                idx += 2;
            }
            "px" => {
                if let Some(KeyExpiry::EX(_)) = args.expiry {
                    return Err(StorageError::CommandSyntaxError(agruments.join(" ")));
                }
                if idx + 1 >= agruments.len() {
                    return Err(StorageError::CommandSyntaxError(agruments.join(" ")));
                }
                let value: u64 = agruments[idx + 1]
                    .parse::<u64>()
                    .map_err(|_| StorageError::CommandSyntaxError(agruments.join(" ")))?;

                args.expiry = Some(KeyExpiry::PX(value));
                idx += 2;
            }
            "get" => {
                args.get = true;
                idx += 1;
            }
            _ => return Err(StorageError::CommandSyntaxError(agruments.join(" "))),
        }
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_nx() {
        let commands: Vec<String> = vec![String::from("nx")];
        let args = parse_set_arguments(&commands).unwrap();
        assert_eq!(args.existence, Some(KeyExistence::NX));
    }
    #[test]
    fn test_parse_nx_lowercase() {
        let commands: Vec<String> = vec![String::from("NX")];
        let args = parse_set_arguments(&commands).unwrap();
        assert_eq!(args.existence, Some(KeyExistence::NX));
    }
}
