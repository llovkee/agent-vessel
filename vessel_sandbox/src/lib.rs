use std::path::Path;
use bollard::models::ContainerCreateBody;
use bollard::Docker;
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::models::HostConfig;
use bollard::query_parameters::{CreateContainerOptionsBuilder, RemoveContainerOptionsBuilder};
use futures_util::{StreamExt};

pub struct SandboxManager {
    docker: Docker,
    container_id: Option<String>,
}

impl SandboxManager {
    pub fn new() -> Result<Self, bollard::errors::Error> {
        let docker = Docker::connect_with_socket_defaults()?;
        Ok(Self{ docker, container_id: None })
    }

    pub async fn spawn(&mut self, image_name: &str, workspace_path: &Path) -> Result<(), bollard::errors::Error> {
        let mount_path = workspace_path.to_str().unwrap();
        let host_config = HostConfig {
            memory: Some(512 * 1024 * 1024),
            cpu_shares: Some(1024),
            network_mode: Some("none".into()),
            binds: Some(vec![format!("{}:/workspace:rw", mount_path)]),
            ..Default::default()
        };


        let config = ContainerCreateBody {
            cmd: Some(vec!["sleep".into(), "infinity".into()]),
            image: Some(image_name.into()),
            working_dir: Some("/workspace".into()),
            host_config: Some(host_config),
            ..Default::default()
        };

        let options = CreateContainerOptionsBuilder::default()
            .name("agent-vessel-sandbox")
            .build();

        let container = self.docker.create_container(Some(options), config).await?;
        self.container_id = Some(container.id.clone());

        self.docker.start_container(&container.id, None).await?;

        Ok(())
    }

    pub async fn execute_bash(&self, command: &str) -> Result<String, bollard::errors::Error> {
        let container_id = self.container_id.as_ref().expect("sandbox is not running");
        let exec_options = CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            cmd: Some(vec!["/bin/bash", "-c", command]),
            ..Default::default()
        };

        let exec = self.docker.create_exec(container_id, exec_options).await?;

        let mut out = String::new();
        if let StartExecResults::Attached { output: mut stream, ..} = self.docker.start_exec(&exec.id, None).await? {
            while let Some(Ok(msg)) = stream.next().await {
                out.push_str(&String::from_utf8_lossy(&msg.into_bytes()))
            }
        }
        Ok(out)
    }

    pub async fn purge(&mut self) -> Result<(), bollard::errors::Error> {
        if let Some(id) = self.container_id.take() {
            let rm_opt = RemoveContainerOptionsBuilder::default()
                .force(true)
                .v(true)
                .link(false)
                .build();
            self.docker.remove_container(&id, Some(rm_opt)).await?;
            self.container_id = None;
        }
        Ok(())
    }
}