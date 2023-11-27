
pub trait PlayInfo{
    fn get_current_time_in_sample(&self)->u64;
    fn increment_time(&mut self);
    fn get_channels(&self)->u64;
    fn get_frame_per_buffer(&self)->u64;
    fn get_samplerate(&self)->f64;
}