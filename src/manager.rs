use std::{
    env,
    ffi::CString,
    fs,
    io::{self, Read, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use config_file_structure::FilesInfo;
use nix::{
    sys::wait::waitpid,
    unistd::{execvp, fork, ForkResult},
};

use crate::config::Config;

mod config_file_structure {
    use std::path::PathBuf;

    pub struct FilesInfo {
        path: PathBuf,
        pub sample_config: PathBuf,
        pub config: PathBuf,
        pub runtime: RuntimeDir,
    }

    impl FilesInfo {
        pub fn new(path: &PathBuf) -> Self {
            Self {
                path: path.into(),
                sample_config: path.join("sample_config.yml"),
                config: path.join("config.yml"),
                runtime: RuntimeDir::new(&path.join("runtime")),
            }
        }

        pub fn path(&self) -> PathBuf {
            self.path.clone()
        }
    }

    pub struct RuntimeDir {
        path: PathBuf,
        pub programs: ProgramsDir,
        pub dockerfile: PathBuf,
        pub docker_compose_file: PathBuf,
    }

    impl RuntimeDir {
        pub fn new(path: &PathBuf) -> Self {
            Self {
                path: path.into(),
                programs: ProgramsDir::new(&path.join("programs")),
                dockerfile: path.join("Dockerfile"),
                docker_compose_file: path.join("docker-compose.yml"),
            }
        }

        pub fn path(&self) -> PathBuf {
            self.path.clone()
        }
    }

    pub struct ProgramsDir {
        path: PathBuf,
    }

    impl ProgramsDir {
        pub fn new(path: &PathBuf) -> Self {
            Self { path: path.into() }
        }

        pub fn path(&self) -> PathBuf {
            self.path.clone()
        }
    }
}

pub struct AppManager {
    files_info: FilesInfo,
}

impl AppManager {
    pub fn new(base_path: &PathBuf) -> Self {
        Self {
            files_info: FilesInfo::new(base_path),
        }
    }

    pub fn setup_minimum_requirements(&self) -> Result<()> {
        let is_exists = |path: &PathBuf| fs::metadata(path).is_ok();
        let files = &self.files_info;

        if !is_exists(&files.path()) {
            fs::create_dir(&files.path()).context("Failed to create the config directory.")?;
        }

        // if !is_exists(&files.sample_config_path) {
        //     fs::File::create(&files.sample_config_path)
        //         .context("Failed to create the sample config file.")?;
        // }

        if !is_exists(&files.config) {
            let default_config = Config::default();
            let default_config_yaml = serde_yaml::to_string(&default_config)
                .context("Failed to parse default config to yaml format.")?;
            let mut config_file =
                fs::File::create(&files.config).context("Failed to create the config file.")?;
            config_file
                .write_all(default_config_yaml.as_bytes())
                .context("Failed to write contents to the config file.")?;
        }

        if !is_exists(&files.runtime.path()) {
            fs::create_dir(&files.runtime.path())
                .context("Failed to create the runtime directory.")?;
        }

        Ok(())
    }

    /// open config file. If the config file doesn't exist, creates the new file and its content become default config.
    pub fn open_config(&self) -> Result<Config> {
        let mut config_file =
            fs::File::open(&self.files_info.config).context("Failed to open the config file.")?;
        let mut config_buffer = String::new();
        config_file
            .read_to_string(&mut config_buffer)
            .context("Failed to read contents from the config file")?;
        let config: Config = serde_yaml::from_str(&config_buffer)
            .context("Failed to parse contents of the config file to yaml format.")?;
        Ok(config)
    }

    pub fn init(&mut self, host_dir_path: &PathBuf) -> Result<()> {
        let config = self.open_config().context("Failed to open the config.")?;
        let files = &self.files_info;

        // change the current working directory to the runtime directory.
        env::set_current_dir(&files.runtime.path()).with_context(|| {
            format!(
                "Failed to change current working directory to {}",
                &files.runtime.path().display()
            )
        })?;

        // create a dockerfile
        {
            let mut dockerfile_buffer = config.to_dockerfile();

            let programs_path = files.runtime.programs.path();
            let programs_relative_path_from_runtime_dir = programs_path
                .strip_prefix(&files.runtime.path())
                .context("Failed to get the relative path of the programs directory.")?;
            dockerfile_buffer.push_str(
                format!(
                    "COPY {} /root/workspace\n",
                    &programs_relative_path_from_runtime_dir.display()
                )
                .as_str(),
            );

            dockerfile_buffer.push_str("WORKDIR /root/workspace\n");

            let mut dockerfile = fs::File::create(&files.runtime.dockerfile)
                .context("Failed to create/open the dockerfile.")?;
            dockerfile
                .write_all(dockerfile_buffer.as_bytes())
                .context("Failed to write config to the dockerfile.")?;
        }

        // create a docker-compose.yml
        {
            let docker_compose_buffer = generate_docker_compose_config("Dockerfile", host_dir_path);
            let mut docker_compose_file = fs::File::create(&files.runtime.docker_compose_file)
                .with_context(|| {
                    format!(
                        "Failed to create/open {:?}",
                        files.runtime.docker_compose_file.file_name()
                    )
                })?;
            docker_compose_file
                .write_all(docker_compose_buffer.as_bytes())
                .context("Failed to write config to the docker compose file.")?;
        }

        // copy programs
        {
            if let Err(err) = fs::remove_dir_all(&files.runtime.programs.path()) {
                if err.kind() != io::ErrorKind::NotFound {
                    return Err(err).with_context(|| {
                        format!(
                            "Failed to remove the programs directory {}",
                            &files.runtime.programs.path().display()
                        )
                    })?;
                }
            }
            let mut copy_option = fs_extra::dir::CopyOptions::new();
            copy_option.copy_inside = true;
            copy_option.overwrite = true;
            fs_extra::dir::copy(host_dir_path, &files.runtime.programs.path(), &copy_option)
                .with_context(|| {
                    format!(
                        "Failed to copy {} to {}",
                        host_dir_path.display(),
                        &files.runtime.programs.path().display(),
                    )
                })?;
        }

        // docker compose up -d --build
        {
            match unsafe { fork() }.unwrap() {
                ForkResult::Child => {
                    let cmd = CString::new("docker")?;
                    execvp(
                        &cmd,
                        &vec![
                            &cmd,
                            &CString::new("compose").unwrap(),
                            &CString::new("up").unwrap(),
                            &CString::new("-d").unwrap(),
                            &CString::new("--build").unwrap(),
                        ],
                    )
                    .unwrap();
                }
                ForkResult::Parent { child } => {
                    waitpid(child, None).unwrap();
                }
            }
        }

        // docker compose exec pwn /usr/bin/fish
        {
            let cmd = CString::new("docker")?;
            execvp(
                &cmd,
                &vec![
                    &cmd,
                    &CString::new("compose").unwrap(),
                    &CString::new("exec").unwrap(),
                    &CString::new("pwn").unwrap(),
                    &CString::new("/usr/bin/fish").unwrap(),
                ],
            )
            .unwrap();
        }

        Ok(())
    }

    pub fn enter(&self) -> Result<()> {
        let files = &self.files_info;

        // change the current working directory to the runtime directory.
        env::set_current_dir(&files.runtime.path()).with_context(|| {
            format!(
                "Failed to change current working directory to {}",
                &files.runtime.path().display()
            )
        })?;

        let cmd = CString::new("docker")?;
        execvp(
            &cmd,
            &vec![
                &cmd,
                &CString::new("compose").unwrap(),
                &CString::new("exec").unwrap(),
                &CString::new("pwn").unwrap(),
                &CString::new("/usr/bin/fish").unwrap(),
            ],
        )
        .unwrap();

        Ok(())
    }

    pub fn kill(&self) -> Result<()> {
        let files = &self.files_info;
        // change the current working directory to the runtime directory.
        env::set_current_dir(&files.runtime.path()).with_context(|| {
            format!(
                "Failed to change current working directory to {}",
                &files.runtime.path().display()
            )
        })?;

        match unsafe { fork() }.unwrap() {
            ForkResult::Child => {
                let cmd = CString::new("docker")?;
                execvp(
                    &cmd,
                    &vec![
                        &cmd,
                        &CString::new("compose").unwrap(),
                        &CString::new("kill").unwrap(),
                        &CString::new("pwn").unwrap(),
                    ],
                )
                .unwrap();
            }
            ForkResult::Parent { child } => {
                waitpid(child, None).unwrap();
            }
        }

        let cmd = CString::new("docker")?;
        execvp(
            &cmd,
            &vec![
                &cmd,
                &CString::new("compose").unwrap(),
                &CString::new("rm").unwrap(),
                &CString::new("-f").unwrap(),
                &CString::new("pwn").unwrap(),
            ],
        )
        .unwrap();

        Ok(())
    }
}

fn generate_docker_compose_config(dockerfile_name: &str, host_dir_path: &PathBuf) -> String {
    let template = r#"
version: "3.9"
services:
    pwn:
        build:
            context: .
            dockerfile: {dockerfile}
            args:
                UID: $UID
                GID: $GID
                USERNAME: $USERNAME
                GROUPNAME: $GROUPNAME
        user: $UID:$GID
        tty: true
        privileged: true
        ulimits:
            core:
                soft: -1
                hard: -1
        cap_add:
            - "SYS_PTRACE"
        security_opt:
            - "seccomp=unconfined"
        ports:
            - "127.0.0.1:3333:3333"
        volumes:
            - {host_dir}:/root/workspace:rw
"#;

    template
        .replace("{dockerfile}", dockerfile_name)
        .replace("{host_dir}", &host_dir_path.display().to_string())
}
