use image::ImageFormat;

use crate::Spec;

pub trait Engine {
    fn apply(&mut self, specs: &[Spec]);
    fn process(self, format: ImageFormat) -> Vec<u8>;
}

pub trait SepcTransform<T> {
    fn transform(&mut self, op: T);
}
