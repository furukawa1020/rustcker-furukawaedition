#[cfg(test)]
mod tests {
    use crate::container::{Container, Created, Running, Stopped};


    #[test]
    fn test_create_container() {
        let c = Container::new("test-id".to_string());
        assert_eq!(c.state, Created);
    }

    #[test]
    fn test_valid_lifecycle() {
        let c = Container::new("test-id".to_string());
        let running = c.start().expect("Should be able to start created container");
        let _stopped = running.stop().expect("Should be able to stop running container");
        
        // This line would fail to compile if uncommented, proving strict FSM:
        // stopped.start(); 
    }
}
