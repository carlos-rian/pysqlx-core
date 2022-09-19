pub trait SqlConnecton {
    fn connect(&self) {}
    fn disconnect(&self) {}
    fn query(&self) {}
    fn execute(&self) {}
}