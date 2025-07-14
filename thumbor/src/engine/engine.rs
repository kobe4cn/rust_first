use image::ImageFormat;

use crate::Spec;
use anyhow::Result;

pub trait Engine {
    fn create<T>(data: T) -> Result<Self>
    where
        Self: Sized,
        T: TryInto<Self>,
    {
        data.try_into()
            .map_err(|_| anyhow::anyhow!("Failed to create engine"))
    }

    fn apply(&mut self, specs: &[Spec]);
    fn process(self, format: ImageFormat) -> Vec<u8>;
}

pub trait SepcTransform<T> {
    fn transform(&mut self, op: T);
}
