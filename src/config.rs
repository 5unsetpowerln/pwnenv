use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result};
use dir::home_dir;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    base_image: String,
    init_script: Vec<String>,
    post_script: Vec<String>,
    tools: Vec<Tool>,
}

impl Config {
    fn new(base_image: &str, init_script: &[&str], tools: &[Tool], post_script: &[&str]) -> Self {
        Self {
            base_image: base_image.to_string(),
            init_script: init_script.into_iter().map(|&s| s.to_string()).collect(),
            tools: tools.to_vec(),
            post_script: post_script.into_iter().map(|&s| s.to_string()).collect(),
        }
    }

    pub fn default() -> Self {
        let mut t = IndexMap::<&str, Vec<&str>>::new();
        t.insert(
            "build-essential",
            vec!["RUN apt install build-essential -y"],
        );
        t.insert(
            "ca-certificates",
            vec!["RUN apt install ca-certificates -y"],
        );
        t.insert("libssl-dev", vec!["RUN apt install libssl-dev -y"]);
        t.insert("zlib1g-dev", vec!["RUN apt install zlib1g-dev -y"]);
        t.insert("libbz2-dev", vec!["RUN apt install libbz2-dev -y"]);
        t.insert(
            "libreadline-dev",
            vec!["RUN apt install libreadline-dev -y"],
        );
        t.insert("libsqlite3-dev", vec!["RUN apt install libsqlite3-dev -y"]);
        t.insert("wget", vec!["RUN apt install wget -y"]);
        t.insert("curl", vec!["RUN apt install curl -y"]);
        t.insert("llvm", vec!["RUN apt install llvm -y"]);
        t.insert("make", vec!["RUN apt install make -y"]);
        t.insert("zip", vec!["RUN apt install zip -y"]);
        t.insert("unzip", vec!["RUN apt install unzip -y"]);
        t.insert(
            "libncurses5-dev",
            vec!["RUN apt install libncurses5-dev -y"],
        );
        t.insert(
            "libncursesw5-dev",
            vec!["RUN apt install libncursesw5-dev -y"],
        );
        t.insert("xz-utils", vec!["RUN apt install xz-utils -y"]);
        t.insert("tk-dev", vec!["RUN apt install tk-dev -y"]);
        t.insert("libxml2-dev", vec!["RUN apt install libxml2-dev -y"]);
        t.insert("libxmlsec1-dev", vec!["RUN apt install libxmlsec1-dev -y"]);
        t.insert("libffi-dev", vec!["RUN apt install libffi-dev -y"]);
        t.insert("liblzma-dev", vec!["RUN apt install liblzma-dev -y"]);
        t.insert("libyaml-dev", vec!["RUN apt install libyaml-dev -y"]);
        t.insert("python3", vec!["RUN apt install python3 -y"]);
        t.insert("python3-dev", vec!["RUN apt install python3-dev -y"]);
        t.insert("python3-pip", vec!["RUN apt install python3-pip -y"]);
        t.insert("gcc", vec!["RUN apt install gcc -y"]);
        t.insert("tree", vec!["RUN apt install tree -y"]);
        t.insert("git", vec!["RUN apt install git -y"]);
        t.insert("libyaml-dev", vec!["RUN apt install libyaml-dev -y"]);
        t.insert("neovim", vec!["RUN apt install neovim -y"]);
        t.insert("neofetch", vec!["RUN apt install neofetch -y"]);
        t.insert("openssh-server", vec!["RUN apt install openssh-server -y"]);
        t.insert("patchelf", vec!["RUN apt install patchelf -y"]);
        t.insert("elfutils", vec!["RUN apt install elfutils -y"]);
        t.insert("psmisc", vec!["RUN apt install psmisc -y"]);
        t.insert("file", vec!["RUN apt install file -y"]);
        t.insert("devscripts", vec!["RUN apt install devscripts -y"]);
        t.insert(
            "fish",
            vec![
                "RUN apt install fish -y",
                "RUN chsh -s /bin/fish",
                "RUN mkdir /root/.config/fish",
            ],
        );
        t.insert(
            "seccomp",
            vec!["RUN apt install libseccomp-dev libseccomp2 seccomp -y"],
        );
        t.insert("ruby", vec!["RUN apt install ruby-dev -y"]);
        t.insert(
            "pyenv",
            vec![
                "RUN git clone https://github.com/pyenv/pyenv.git $HOME/.pyenv",
                "RUN echo 'set -x PYENV_ROOT /root/.pyenv' >> /root/.config/fish/config.fish",
                "RUN echo 'set -x PATH  /root/.pyenv/bin $PATH' >> /root/.config/fish/config.fish",
                "RUN echo 'set -x PATH /root/.pyenv/shims $PATH' >> /root/.config/fish/config.fish",
                "RUN /root/.pyenv/bin/pyenv install pypy3.9-7.3.16",
                "RUN /root/.pyenv/bin/pyenv global pypy3.9-7.3.16",
                "ENV PATH $PATH:/root/.pyenv/shims/",
            ],
        );
        t.insert("ptrlib", vec!["RUN pip install ptrlib"]);
        t.insert("pwntools", vec!["RUN pip install pwntools"]);
        t.insert(
            "rust",
            vec![
                "RUN curl https://sh.rustup.rs -sSf | sh -s -- -y",
                "RUN echo 'set -x PATH /root/.cargo/bin $PATH' >> /root/.config/fish/config.fish",
                "ENV PATH $PATH:/root/.cargo/bin",
            ],
        );
        t.insert("ropr", vec!["RUN cargo install ropr"]);
        t.insert(
                "bat",
                vec![
                    "RUN cargo install bat",
                    "RUN echo \'alias bat=\"bat --theme=gruvbox-dark\"\' >> /root/.config/fish/config.fish",
                ],
            );
        t.insert(
            "eza",
            vec![
                "RUN cargo install eza",
                "RUN echo \'alias ls=\"eza\"\' >> /root/.config/fish/config.fish",
            ],
        );
        t.insert(
            "gdb",
            vec![
                //gdb
                "RUN apt install gdb -y",
                // pwndbg
                "WORKDIR /root/tools",
                "RUN git clone https://github.com/pwndbg/pwndbg",
                "WORKDIR /root/tools/pwndbg",
                "RUN ./setup.sh",
                // ptr
                "RUN echo '#!/bin/bash' > /usr/local/bin/ptr",
                "RUN echo 'gdb -q -p $(pidof $1)' >> /usr/local/bin/ptr",
                "RUN echo 'set follow-fork-mode parent' >> /root/.gdbinit",
            ],
        );
        t.insert("pwninit", vec!["RUN cargo install pwninit"]);
        t.insert(
            "seccomp-tools",
            vec!["RUN gem install seccomp-tools --no-document --force"],
        );
        t.insert(
            "glibc",
            vec![
                // glibc source code
                "WORKDIR /root/",
                "RUN git clone https://github.com/bminor/glibc",
                "RUN ln -s /root/glibc /root/workspace/glibc",
                // glibc all in one
                "WORKDIR /root/",
                "RUN git clone https://github.com/matrix1001/glibc-all-in-one",
                "RUN ln -s /root/glibc-all-in-one /root/workspace/glibc-all-in-one",
            ],
        );
        t.insert(
            "ripgrep",
            vec![
                "RUN apt install ripgrep -y",
                "RUN echo 'alias grep=\"rg\"\' >> /root/.config/fish/config.fish",
            ],
        );

        let mut tools = Vec::new();
        for (name, script) in t {
            tools.push(Tool::new(name, &[ToolInstall::new("default", &script)]));
        }

        let init_script = [
            "ENV DEBIAN_FRONTEND noninteractive",
            "ENV TZ Asia/Tokyo",
            "RUN apt update",
            "RUN apt-get update",
            "RUN apt install -y",
            "RUN mkdir /root/tools",
            "RUN mkdir /root/workspace",
            "RUN mkdir /root/.config",
        ];

        let post_script =
            ["RUN echo \"set -x LC_CTYPE C.UTF-8\" >> /root/.config/fish/config.fish"];

        Config::new("amd64/ubuntu:22.04", &init_script, &tools, &post_script)
    }

    pub fn to_dockerfile(&self) -> String {
        let mut dockerfile = String::new();

        dockerfile.push_str(format!("FROM {}\n", self.base_image).as_str());

        for init_script_line in self.init_script.iter() {
            dockerfile.push_str(format!("{}\n", init_script_line).as_str());
        }

        for tool in self.tools.iter() {
            let optimized_install = tool
                .installs
                .iter()
                .find(|&i| i.base_image == self.base_image);
            let default_install = tool
                .installs
                .iter()
                .find(|&i| i.base_image == "default".to_string());

            if optimized_install.is_some() || default_install.is_some() {
                if let Some(o_install) = optimized_install {
                    for script_line in o_install.script.iter() {
                        dockerfile.push_str(format!("{}\n", script_line).as_str());
                    }
                    continue;
                }

                if let Some(d_install) = default_install {
                    for script_line in d_install.script.iter() {
                        dockerfile.push_str(format!("{}\n", script_line).as_str());
                    }
                    continue;
                }
            }
        }

        for post_script_line in self.post_script.iter() {
            dockerfile.push_str(format!("{}\n", post_script_line).as_str());
        }

        dockerfile
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Tool {
    name: String,
    installs: Vec<ToolInstall>,
}

impl Tool {
    pub fn new(name: &str, installs: &[ToolInstall]) -> Self {
        Self {
            name: name.to_string(),
            installs: installs.to_vec(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ToolInstall {
    base_image: String,
    script: Vec<String>,
}

impl ToolInstall {
    pub fn new(base_image: &str, script: &[&str]) -> Self {
        Self {
            base_image: base_image.to_string(),
            script: script.into_iter().map(|&s| s.to_string()).collect(),
        }
    }
}
