use clap::Parser;
use ssh_rs::{Session, ssh};
use ssh_rs::key_pair::KeyPairType;
use std::path::PathBuf;

#[derive(Debug)]
struct ParseOciEngineError;

impl std::fmt::Display for ParseOciEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid value, values are podman or docker")
    }
}

impl std::error::Error for ParseOciEngineError {}

#[derive(PartialEq, Debug, Clone)]
enum OciEngine {
    Podman,
    Docker,
}


impl std::fmt::Display for OciEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            OciEngine::Podman => "podman",
            OciEngine::Docker => "docker",
        })
    }
}

impl std::str::FromStr for OciEngine {
    type Err = ParseOciEngineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "podman" => Ok(OciEngine::Podman),
            "docker" => Ok(OciEngine::Docker),
            _ => Err(ParseOciEngineError),
        }
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    host: String,
    #[clap(short, long, value_parser)]
    user: String,
    #[clap(short, long, value_parser, default_value_t = 22)]
    port: u16,

    #[clap(short, long, value_parser, default_value_t = OciEngine::Podman)]
    engine: OciEngine,

    #[clap(short = 'P', long, value_parser)]
    password: Option<String>,

    #[clap(short, long, value_parser)]
    key_path: Option<String>,

    #[clap(short, long, value_parser, default_value = ".")]
    target_dir: String,
}


struct Target {
    host: String,
    port: u16,
}
impl Target {
    pub fn new(host: String, port: u16) -> Self {
        Target { host, port }
    }
}

impl std::string::ToString for Target {
    fn to_string(&self) -> String {
        return format!("{}:{}", self.host, self.port);
    }
}

struct Volume {
    name: String,
}

fn get_volumes(engine: &OciEngine, session: &mut Session) -> ssh_rs::error::SshResult<Vec<Volume>> {
    let exec = session.open_exec()?;
    let res = exec.send_command(&(engine.to_string() + " volume ls -f 'driver=local' --format '{{.Name}}'"))?;
    let lines = String::from_utf8(res).expect("bad result");
    return Ok(lines.lines().map(|name| {
        Volume {
            name: name.to_owned(),
        }
    }).collect());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut session: Session = ssh::create_session();
    let user = args.user;
    let engine = args.engine;
    let target_dir = args.target_dir.parse::<PathBuf>().unwrap();
    std::fs::create_dir_all(&target_dir)?;

    if let Some(password) = args.password {
        session.set_user_and_password(user, password);
    } else {
        // pem format key string:
        //      -----BEGIN RSA PRIVATE KEY-----
        //          xxxxxxxxxxxxxxxxxxxxx
        //      -----END RSA PRIVATE KEY-----
        // KeyPairType::SshRsa -> Rsa type algorithm, currently only supports rsa.
        let pem = {
            let default_path = "~/.ssh/id_rsa.pem".to_owned();
            let key_path = args.key_path.unwrap_or(default_path);
            std::fs::read_to_string(key_path).expect("invalid pem path")
        };
        session.set_user_and_key_pair(user, pem, KeyPairType::SshRsa).unwrap();
    }

    let target = Target::new(args.host.clone(), args.port);
    session.connect(target.to_string())?;
    for volume in get_volumes(&engine, &mut session)? {

        println!("backing up {}", volume.name);
        let backup_file = target_dir.join(format!("{}.tar.gz", volume.name));
        let command = format!("{} volume export \"{}\" | gzip", engine, volume.name);
        let exec = session.open_exec()?;
        let result = exec.send_command(&command)?;
        std::fs::write(backup_file ,result)?;
        println!("done");
    }

    Ok(())
}
