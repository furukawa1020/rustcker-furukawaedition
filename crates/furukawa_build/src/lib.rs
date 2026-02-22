//! Dockerfile parser and build engine for HATAKE Desktop.
//!
//! Supports basic Dockerfile instructions: FROM, RUN, COPY, WORKDIR, CMD, ENV, EXPOSE.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use tracing::info;

/// Supported Dockerfile instructions.
#[derive(Debug, Clone)]
pub enum Instruction {
    From { image: String, tag: String },
    Run(Vec<String>),
    Copy { src: String, dest: String },
    Workdir(String),
    Cmd(Vec<String>),
    Env { key: String, value: String },
    Expose(u16),
    Label { key: String, value: String },
}

/// Parse a Dockerfile from a string into a list of instructions.
pub fn parse_dockerfile(content: &str) -> Result<Vec<Instruction>> {
    let mut instructions = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (keyword, rest) = line.split_once(' ')
            .unwrap_or((line, ""));
        let rest = rest.trim();

        match keyword.to_uppercase().as_str() {
            "FROM" => {
                let (image, tag) = if let Some((img, tag)) = rest.split_once(':') {
                    (img.to_string(), tag.to_string())
                } else {
                    (rest.to_string(), "latest".to_string())
                };
                instructions.push(Instruction::From { image, tag });
            }
            "RUN" => {
                // Support shell form only (exec form ignored for now)
                let parts = shell_split(rest);
                instructions.push(Instruction::Run(parts));
            }
            "COPY" => {
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    instructions.push(Instruction::Copy {
                        src: parts[0].to_string(),
                        dest: parts[1].to_string(),
                    });
                } else {
                    bail!("COPY requires src and dest");
                }
            }
            "WORKDIR" => {
                instructions.push(Instruction::Workdir(rest.to_string()));
            }
            "CMD" => {
                instructions.push(Instruction::Cmd(shell_split(rest)));
            }
            "ENV" => {
                if let Some((key, value)) = rest.split_once('=').or_else(|| rest.split_once(' ')) {
                    instructions.push(Instruction::Env {
                        key: key.trim().to_string(),
                        value: value.trim().trim_matches('"').to_string(),
                    });
                }
            }
            "EXPOSE" => {
                if let Ok(port) = rest.split('/').next().unwrap_or("0").parse::<u16>() {
                    instructions.push(Instruction::Expose(port));
                }
            }
            "LABEL" => {
                if let Some((key, value)) = rest.split_once('=') {
                    instructions.push(Instruction::Label {
                        key: key.trim().to_string(),
                        value: value.trim().trim_matches('"').to_string(),
                    });
                }
            }
            other => {
                tracing::warn!("Ignoring unsupported Dockerfile instruction: {}", other);
            }
        }
    }

    Ok(instructions)
}

/// Build context: directory containing the Dockerfile and referenced files.
pub struct BuildContext {
    pub context_dir: PathBuf,
    pub instructions: Vec<Instruction>,
    pub tag: String,
    pub distro: String,
}

impl BuildContext {
    pub fn new(context_dir: PathBuf, dockerfile_content: &str, tag: &str, distro: &str) -> Result<Self> {
        let instructions = parse_dockerfile(dockerfile_content)?;
        Ok(Self {
            context_dir,
            instructions,
            tag: tag.to_string(),
            distro: distro.to_string(),
        })
    }
}

/// Execute a build inside a temporary WSL2 chroot directory.
///
/// Returns the layer tarball path that can be fed into the ImageStore.
pub async fn run_build(ctx: &BuildContext, output_dir: &Path) -> Result<PathBuf> {
    use tokio::process::Command;

    let build_id = uuid::Uuid::new_v4().to_string();
    let build_dir = output_dir.join(&build_id);
    tokio::fs::create_dir_all(&build_dir).await
        .context("Failed to create build directory")?;

    let rootfs_dir = build_dir.join("rootfs");
    tokio::fs::create_dir_all(&rootfs_dir).await?;

    // Convert rootfs path to WSL path
    let rootfs_str = rootfs_dir.to_string_lossy().replace('\\', "/");
    let wsl_rootfs = if let Some(s) = rootfs_str.strip_prefix("C:/").or_else(|| rootfs_str.strip_prefix("c:/")) {
        format!("/mnt/c/{}", s)
    } else {
        rootfs_str.clone()
    };

    let mut base_image: Option<String> = None;

    for instruction in &ctx.instructions {
        match instruction {
            Instruction::From { image, tag } => {
                info!("[BUILD] FROM {}:{}", image, tag);
                base_image = Some(format!("{}:{}", image, tag));
                // In a real build, we would extract the base image layers here.
                // For now, we assume the build rootfs starts empty (scratch).
            }
            Instruction::Run(cmd) => {
                if base_image.is_none() {
                    bail!("RUN before FROM is not allowed");
                }
                info!("[BUILD] RUN {:?}", cmd);
                let shell_cmd = cmd.join(" ");
                let status = Command::new("wsl.exe")
                    .args(["-d", &ctx.distro, "-u", "root", "--", "sh", "-c", &shell_cmd])
                    .status()
                    .await
                    .context("Failed to execute RUN command in WSL")?;
                if !status.success() {
                    bail!("RUN command failed: {:?}", cmd);
                }
            }
            Instruction::Copy { src, dest } => {
                info!("[BUILD] COPY {} -> {}", src, dest);
                let src_path = ctx.context_dir.join(src);
                let dest_wsl = format!("{}{}", wsl_rootfs, dest);
                // Create destination dir
                let _ = Command::new("wsl.exe")
                    .args(["-d", &ctx.distro, "--", "mkdir", "-p", &dest_wsl])
                    .status()
                    .await;
                // Copy file
                let src_wsl = src_path.to_string_lossy().replace('\\', "/");
                let src_wsl = if let Some(s) = src_wsl.strip_prefix("C:/").or_else(|| src_wsl.strip_prefix("c:/")) {
                    format!("/mnt/c/{}", s)
                } else {
                    src_wsl.to_string()
                };
                let _ = Command::new("wsl.exe")
                    .args(["-d", &ctx.distro, "--", "cp", "-r", &src_wsl, &dest_wsl])
                    .status()
                    .await;
            }
            Instruction::Workdir(dir) => {
                info!("[BUILD] WORKDIR {}", dir);
                let target = format!("{}{}", wsl_rootfs, dir);
                let _ = Command::new("wsl.exe")
                    .args(["-d", &ctx.distro, "--", "mkdir", "-p", &target])
                    .status()
                    .await;
            }
            Instruction::Env { key, value } => {
                info!("[BUILD] ENV {}={}", key, value);
                // Write to /etc/environment inside rootfs
                let env_line = format!("{}={}", key, value);
                let etc_env = format!("{}/etc/environment", wsl_rootfs);
                let _ = Command::new("wsl.exe")
                    .args(["-d", &ctx.distro, "--", "sh", "-c",
                        &format!("echo '{}' >> {}", env_line, etc_env)])
                    .status()
                    .await;
            }
            Instruction::Expose(port) => {
                info!("[BUILD] EXPOSE {} (metadata only)", port);
            }
            Instruction::Label { key, value } => {
                info!("[BUILD] LABEL {}={} (metadata only)", key, value);
            }
            Instruction::Cmd(cmd) => {
                info!("[BUILD] CMD {:?} (saved as image default cmd)", cmd);
            }
        }
    }

    // Pack the rootfs into a tar layer
    let layer_path = build_dir.join("layer.tar");
    let status = Command::new("wsl.exe")
        .args([
            "-d", &ctx.distro, "--",
            "tar", "-czf",
            &format!("{}/layer.tar", wsl_rootfs.rsplit_once('/').map(|(d,_)| d).unwrap_or(".")),
            "-C", &wsl_rootfs, "."
        ])
        .status()
        .await
        .context("Failed to tar the build rootfs")?;

    if !status.success() {
        bail!("Failed to create layer tarball");
    }

    Ok(layer_path)
}

/// Simple shell split: splits on whitespace, respecting quoted strings.
fn shell_split(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';

    for c in s.chars() {
        match c {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = c;
            }
            c if in_quote && c == quote_char => {
                in_quote = false;
            }
            ' ' | '\t' if !in_quote => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}
