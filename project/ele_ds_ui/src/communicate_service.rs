struct CommunicateService {
    service_ip: String,
    service_port: u16,
}

impl CommunicateService {
    pub fn new(service_ip: String, service_port: u16) -> Self {
        Self {
            service_ip,
            service_port,
        }
    }
    
    pub fn connect(&self) -> Result<(), Box<dyn std::error::Error>>{
        // connect to the service
        
        Ok(())
    }
}