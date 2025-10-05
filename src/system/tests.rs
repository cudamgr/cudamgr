#[cfg(test)]
mod tests {
    use crate::system::*;

    #[test]
    fn test_system_checker_trait() {
        let _checker = DefaultSystemChecker;
        // Just test that the struct can be created
        assert!(true);
    }
}