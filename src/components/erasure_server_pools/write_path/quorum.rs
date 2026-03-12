#[derive(Debug, Clone, Default)]
pub struct WriteQuorum {
    required_data: usize,
    required_total: usize,
    successful_data: usize,
    successful_total: usize,
}

impl WriteQuorum {
    pub fn new(required_data: usize, required_total: usize) -> Self {
        Self {
            required_data,
            required_total,
            successful_data: 0,
            successful_total: 0,
        }
    }

    pub fn record_success(&mut self, is_data_shard: bool) {
        self.successful_total += 1;
        if is_data_shard {
            self.successful_data += 1;
        }
    }

    pub fn is_satisfied(&self) -> bool {
        self.successful_data >= self.required_data && self.successful_total >= self.required_total
    }
}
