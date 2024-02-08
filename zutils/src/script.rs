use crate::{commands, utils};
use anyhow::{anyhow, Result};
use home;
use reqwest;
use std::collections::HashMap;
use std::env;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

pub struct Context {
    /// Dry run or really execute?
    pub really_execute: bool,
    pub vars: HashMap<String, String>,
    pub append_paths: Vec<String>,
    pub os_params: HashMap<String, String>,
    pub arch: String,
}

impl Context {
    pub async fn new(really_execute: bool) -> Result<Self> {
        let os_params = Context::get_os_params().await?;
        let arch = Context::get_arch().await?;
        Ok(Self {
            really_execute,
            append_paths: Vec::new(),
            vars: HashMap::new(),
            arch,
            os_params,
        })
    }

    pub async fn get_arch() -> Result<String> {
        let mut cmd = commands::CommandBuilder::new();
        cmd.cmd("arch", &vec![]).silent();
        let result = cmd.run_for_output().await?.sanitise_stdout()?;
        Ok(result)
    }

    pub async fn get_os_params() -> Result<HashMap<String, String>> {
        let contents = fs::read_to_string("/etc/os-release").await?;
        let mut result = HashMap::new();
        // Read and parse the os_params() file.
        let lines = contents.split("\n");
        for l in lines {
            // Split at =
            if let Some(v) = l.find('=') {
                let variable = &l[0..v];
                let val = &l[v + 1..];
                result.insert(variable.to_string(), val.to_string());
            }
        }
        Ok(result)
    }

    pub fn add_to_path(&mut self, path_str: &str) {
        self.append_paths.push(path_str.to_string());
    }

    pub fn add_to_env(&mut self, name: &str, val: &str) {
        self.vars.insert(name.to_string(), val.to_string());
    }

    pub async fn modify_context(&self, builder: &mut commands::CommandBuilder) -> Result<()> {
        let current_path = env::var("PATH")?;
        let mut my_path: Vec<String> = vec![current_path.to_string()];
        for path in &self.append_paths {
            my_path.push(path.to_string());
        }
        builder.env_var("PATH", &my_path.join(":"));
        for (k, v) in &self.vars {
            builder.env_var(k.as_str(), v.as_str());
        }
        Ok(())
    }

    pub async fn apt_update(&self) -> Result<()> {
        let mut cmd = Command::as_root(&vec!["apt", "update"])?;
        self.modify_context(&mut cmd.cmd).await?;
        cmd.execute(self).await?;
        Ok(())
    }

    pub async fn apt_upgrade(&self) -> Result<()> {
        let mut cmd = Command::as_root(&vec!["apt", "dist-upgrade"])?;
        self.modify_context(&mut cmd.cmd).await?;
        cmd.execute(self).await?;
        Ok(())
    }

    pub async fn apt_remove(&self, pkgs: &Vec<&str>) -> Result<()> {
        let mut fst = vec!["apt", "remove", "-q", "-y"];
        fst.extend(pkgs);
        let mut the_cmd = Command::as_root(&fst)?;
        the_cmd
            .builder()
            .env_var("DEBIAN_FRONTEND", "noninteractive");
        self.modify_context(&mut the_cmd.cmd).await?;
        the_cmd.execute(self).await?;
        Ok(())
    }

    pub async fn install_keyring(&self, url: &str, name: &str) -> Result<()> {
        let dir_path = PathBuf::from("/etc/apt/keyrings");
        if !dir_path.is_dir() {
            fs::create_dir_all(&dir_path).await?;
        }
        let mut name_path = dir_path.clone();
        name_path.push(name);

        if !name_path.exists() {
            println!("Downloading keyring {name} from {url} .. ");
            let body = reqwest::get(url).await?.text().await?;
            let name_path = utils::string_from_path(&name_path)?;
            let mut cmd = commands::CommandBuilder::new();
            cmd.cmd("gpg", &vec!["--dearmor", "-o", &name_path]);
            cmd.run_logged_with_input(&body).await?;
            fs::set_permissions(name_path, std::fs::Permissions::from_mode(0o644)).await?;
        }
        Ok(())
    }

    pub async fn apt_install(&self, pkgs: &Vec<&str>) -> Result<()> {
        let mut fst = vec!["apt", "install", "-q", "-y"];
        fst.extend(pkgs);
        let mut the_cmd = Command::as_root(&fst)?;
        the_cmd
            .builder()
            .env_var("DEBIAN_FRONTEND", "noninteractive");
        self.modify_context(&mut the_cmd.cmd).await?;
        the_cmd.execute(self).await?;
        Ok(())
    }

    pub async fn as_root(&self, cmd: &Vec<&str>) -> Result<()> {
        let mut cmd = Command::as_root(cmd)?;
        self.modify_context(&mut cmd.cmd).await?;
        cmd.execute(self).await?;
        Ok(())
    }

    pub async fn append_bashrc(&self, id: &str, what: &Vec<&str>) -> Result<()> {
        let mut bashrc = home::home_dir().ok_or(anyhow!("Can't get your home directory"))?;
        bashrc.push(".bashrc");
        // Is this already in bashrc
        let mut contents = fs::read_to_string(&bashrc).await?;
        // This is crude, and doesn't account for conditionals, but.
        let id_begin_str = format!("# zws_auto begin {id}");
        let id_end_str = format!("# zws_auto end {id}");
        let mut to_insert = String::new();
        for line in what {
            to_insert.push_str(line);
            to_insert.push_str("\n");
        }
        let mut inserted = false;
        if let Some(start_val) = contents.find(&id_begin_str) {
            if let Some(end_val) = contents.find(&id_end_str) {
                if end_val > start_val {
                    // got it! + 1 for the '\n'.
                    let end_of_start = start_val + id_begin_str.len() + 1;
                    contents.replace_range(end_of_start..end_val, &to_insert);
                    inserted = true;
                }
            }
        }
        if !inserted {
            // Otherwise, it's not there - append it.
            contents.push_str("\n");
            contents.push_str(&id_begin_str);
            contents.push_str("\n");
            contents.push_str(&to_insert);
            contents.push_str(&id_end_str);
            contents.push_str("\n");
        }
        let mut f = File::create(&bashrc).await?;
        f.write_all(contents.as_bytes()).await?;
        Ok(())
    }

    pub async fn shell(&self, cmd: &str) -> Result<()> {
        // "bash" so that we re-source .bashrc
        // the -i here makes the shell interactive - the default .bashrc on Ubuntu refuses to do
        // anything otherwise. We need to source .bashrc or things like nvm won't be added to the path.
        let mut cmd = Command::build("bash", &vec!["-c", cmd])?;
        self.modify_context(&mut cmd.cmd).await?;
        cmd.execute(self).await?;
        Ok(())
    }

    pub async fn gcloud_copy(&self, project: &str, zone: &str, src: &str, tgt: &str) -> Result<()> {
        let mut cmd = Command::build(
            "gcloud",
            &vec![
                "compute",
                "scp",
                "--project",
                project,
                "--zone",
                zone,
                "--tunnel-through-iap",
                src,
                tgt,
            ],
        )?;
        cmd.execute(self).await?;
        Ok(())
    }
}

pub struct Command {
    pub mandatory: bool,
    pub imperative: bool,
    pub cmd: commands::CommandBuilder,
}

impl Command {
    pub fn new() -> Result<Self> {
        Ok(Self {
            mandatory: false,
            imperative: false,
            cmd: commands::CommandBuilder::new(),
        })
    }

    pub fn as_root(args: &Vec<&str>) -> Result<Self> {
        let mut cmd = commands::CommandBuilder::new();
        cmd.cmd("sudo", args);
        Ok(Self {
            mandatory: true,
            imperative: true,
            cmd,
        })
    }

    pub fn build(cmd_str: &str, args: &Vec<&str>) -> Result<Self> {
        let mut cmd = commands::CommandBuilder::new();
        cmd.cmd(cmd_str, args);
        Ok(Self {
            mandatory: true,
            imperative: true,
            cmd,
        })
    }

    pub async fn execute(&mut self, ctx: &Context) -> Result<commands::CommandOutput> {
        if ctx.really_execute {
            Ok(self.cmd.run_logged().await?)
        } else {
            println!("{0}", self.cmd.describe_command()?);
            Ok(commands::CommandOutput::fake(true))
        }
    }

    pub fn builder(&mut self) -> &mut commands::CommandBuilder {
        &mut self.cmd
    }
    pub fn mandatory(&mut self) -> &mut Self {
        self.mandatory = true;
        self
    }
    pub fn imperative(&mut self) -> &mut Self {
        self.imperative = true;
        self
    }
    pub fn interrogative(&mut self) -> &mut Self {
        self.imperative = false;
        self
    }
    pub fn optional(&mut self) -> &mut Self {
        self.mandatory = false;
        self
    }
}
