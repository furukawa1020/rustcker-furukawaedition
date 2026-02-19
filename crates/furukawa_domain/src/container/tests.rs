#[cfg(test)]
mod tests {
    use crate::container::{Container, Created, Running, Config};
    use crate::container::runtime::ContainerRuntime;
    use furukawa_common::Result;
    use async_trait::async_trait;

    struct MockRuntime;

    #[async_trait]
    impl ContainerRuntime for MockRuntime {
         async fn start(&self, _container: &Container<Created>) -> Result<Running> {
             Ok(Running {
                 pid: 1234,
                 started_at: time::OffsetDateTime::now_utc(),
             })
         }
         async fn stop(&self, _container: &Container<Running>) -> Result<()> {
             Ok(())
         }
    }

    #[test]
    fn test_create_container() {
        let config = Config {
            image: "test-image".to_string(),
            cmd: vec!["test-cmd".to_string()],
        };
        let c = Container::new("test-id".to_string(), config);
        assert_eq!(*c.state(), Created);
    }

    #[tokio::test]
    async fn test_valid_lifecycle() {
        let config = Config {
            image: "test-image".to_string(),
            cmd: vec!["test-cmd".to_string()],
        };
        let c = Container::new("test-id".to_string(), config);
        let runtime = MockRuntime;
        let running = c.start(&runtime).await.expect("Should be able to start created container");
        let _stopped = running.stop(&runtime).await.expect("Should be able to stop running container");
        
        // This line would fail to compile if uncommented, proving strict FSM:
        // stopped.start(); 
    }
}
