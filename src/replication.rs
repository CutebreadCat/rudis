use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};

#[derive(Debug, PartialEq, Clone)]
pub struct MasterConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReplicationConfig {
    pub master: Option<MasterConfig>,
    pub reolid: String,
    pub repl_offset: usize,
}
impl ReplicationConfig {
    pub fn new_master() -> Self {
        ReplicationConfig {
            master: None,
            reolid: Alphanumeric.sample_string(&mut rng(), 40),
            repl_offset: 0,
        }
    }
    pub fn new_replica(master_host: String, master_port: u16) -> Self {
        ReplicationConfig {
            master: Some(MasterConfig {
                host: master_host,
                port: master_port,
            }),
            reolid: Alphanumeric.sample_string(&mut rng(), 40),
            repl_offset: 0,
        }
    }
    pub fn info(&self) -> ReplicationInfo {
        ReplicationInfo {
            role: match self.master {
                Some(_) => Role::Replica,
                None => Role::Master,
            },
            reolid: self.reolid.clone(),
            repl_offset: self.repl_offset,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Role {
    Master,
    Replica,
}

#[derive(Debug, PartialEq)]
pub struct ReplicationInfo {
    pub role: Role,
    pub reolid: String,
    pub repl_offset: usize,
}
