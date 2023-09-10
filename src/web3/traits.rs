use crate::state::entities::{Device, Dataset};

pub trait Timestamper {
    fn register_device(&self, device: &Device);
    fn register_dataset(&self, dataset: &Dataset);
}