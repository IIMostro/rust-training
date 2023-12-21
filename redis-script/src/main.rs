extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use colored::{ColoredString, Colorize};
use redis::{AsyncCommands, ConnectionInfo, IntoConnectionInfo, RedisResult};

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let result: ColoredString = match opts.subcommand {
        SubCommand::Keys(ref args) => {
            keys(&get_redis_client().await?, args.key.clone()).await?.join("\n").cyan()
        }
        SubCommand::Delete { key, file } => {
            let mut file_keys = match file {
                None => { vec![] }
                Some(path) => { read_file_keys(path).await? }
            };
            match key {
                None => {}
                Some(k) => {
                    file_keys.push(k);
                }
            };
            delete(&get_redis_client().await?, &file_keys).await?;
            file_keys.join("\n").cyan()
        }
        SubCommand::Config(ref args) => {
            match &args.args {
                None => {
                    let file = File::open("/etc/redis-script/env.toml");
                    match file {
                        Ok(mut f) => {
                            let mut str = String::new();
                            f.read_to_string(&mut str).unwrap();
                            str.cyan()
                        }
                        Err(_) => {
                            "no config".cyan()
                        }
                    }
                }
                Some(context) => {
                    let mut map = HashMap::new();
                    for s in context {
                        let kv: Vec<&str> = s.split("=").collect();
                        if kv.len() == 2 {
                            map.insert(kv[0].to_string(), kv[1].to_string());
                        }
                    }
                    let config = Config::new(SingleRedisProperties::new(map));
                    let config_str = toml::to_string_pretty(&config)?;
                    let _ = fs::create_dir_all("/etc/redis-script");
                    let mut file = File::create("/etc/redis-script/env.toml")?;
                    let _ = file.write_all(&config_str.as_bytes());
                    config_str.cyan()
                }
            }
        }
    };
    println!("{}", result);
    Ok(())
}

async fn read_file_keys(path: String) -> Result<Vec<String>> {
    let mut file = File::open(path).unwrap();
    let mut str = String::new();
    file.read_to_string(&mut str).unwrap();
    let keys: Vec<String> = str.split("\n").map(|x| x.trim()).map(|s| s.to_string()).collect();
    Ok(keys)
}

async fn get_redis_client() -> Result<redis::Client> {
    let env = get_env().await?;
    if let Some(single) = env.single {
        let client = redis::Client::open::<SingleRedisProperties>(single.into())?;
        Ok(client)
    } else {
        panic!("no redis config")
    }
}

async fn get_env() -> Result<Config> {
    let mut file = match File::open("/etc/redis-script/env.toml") {
        Ok(f) => f,
        Err(_) => panic!("please execute:[redis reset config]")
    };
    let mut str = String::new();
    file.read_to_string(&mut str).unwrap();
    let config: Config = match toml::from_str(&str) {
        Ok(c) => c,
        Err(e) => panic!("parse error: {}", e)
    };
    Ok(config)
}

#[derive(Deserialize, Debug, Parser, Serialize)]
struct SingleRedisProperties {
    host: Option<String>,
    port: Option<u16>,
    db: Option<u8>,
    password: Option<String>,
}

impl SingleRedisProperties {
    fn new(param: HashMap<String, String>) -> Self {
        SingleRedisProperties {
            host: param.get("host").map(|s| s.to_string()),
            port: param.get("port").map(|s| u16::from_str(s).unwrap()),
            db: param.get("db").map(|s| u8::from_str(s).unwrap()),
            password: param.get("password").map(|s| s.to_string()),
        }
    }
}

impl IntoConnectionInfo for SingleRedisProperties {
    fn into_connection_info(self) -> RedisResult<ConnectionInfo> {
        let mut str = String::new();
        str.push_str("redis://");
        if let Some(password) = self.password {
            str.push_str(&password);
            str.push_str("@");
        }
        match self.host {
            None => {
                str.push_str("127.0.0.1");
                str.push_str(":");
            }
            Some(host) => {
                str.push_str(&host);
                str.push_str(":");
            }
        };
        match self.port {
            None => {
                str.push_str("6379");
            }
            Some(port) => {
                str.push_str(&port.to_string());
            }
        };
        match self.db {
            None => {}
            Some(db) => {
                str.push_str("/");
                str.push_str(&db.to_string());
            }
        };
        str.into_connection_info()
    }
}

#[derive(Deserialize, Debug, Serialize)]
struct Config {
    single: Option<SingleRedisProperties>,
}

impl Config {
    fn new(properties: SingleRedisProperties) -> Self {
        Config {
            single: Some(properties),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(version = "0.1.0", author = "li.bowei")]
struct Opts {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, Parser)]
enum SubCommand {
    Keys(Keys),
    Delete {
        #[clap(short, long, value_name = "KEYS", help = "redis keys")]
        key: Option<String>,
        #[clap(short, long, value_name = "FILE", help = "redis keys file")]
        file: Option<String>,
    },
    Config(ConfigArgs),
}

#[derive(Debug, Parser)]
struct ConfigArgs {
    args: Option<Vec<String>>,
}

#[derive(Debug, Parser)]
struct Keys {
    key: String,
}


async fn keys(client: &redis::Client, key: String) -> Result<Vec<String>> {
    let keys: Vec<String> = client.get_async_connection().await?.keys(key).await?;
    Ok(keys)
}

async fn delete(client: &redis::Client, key: &Vec<String>) -> Result<String> {
    println!("delete key: {:?}", key.join("\n"));
    let mut conn = client.get_async_connection().await?;
    conn.del(&key).await?;
    Ok("ok".to_string())
}

#[cfg(test)]
mod tests {
    use crate::{delete, get_redis_client, keys, read_file_keys};

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_keys_should_work() {
        let client = get_redis_client().await.unwrap();
        let keys = keys(&client, "ax-process:*".to_string()).await.unwrap();
        println!("{:?}", keys);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_delete_should_work() {
        let client = get_redis_client().await.unwrap();
        let keys = keys(&client, "ax-process:*".to_string()).await.unwrap();
        delete(&client, &keys).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_open_file_should_work() {
        let keys = read_file_keys("d:\\1.txt".to_string()).await?;
        println!("{:?}", keys);
    }
}