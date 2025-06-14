use std::io::Cursor;

use anyhow::Error;
use bytes::Bytes;
use image::{DynamicImage, ImageBuffer, ImageFormat};
use lazy_static::lazy_static;
use photon_rs::{
    PhotonImage,
    effects::adjust_contrast,
    filters,
    multiple::watermark,
    native::open_image_from_bytes,
    transform::{self, SamplingFilter},
};

lazy_static! {
    static ref WATERMARK_FONT: PhotonImage = {
        let data = include_bytes!("../../images/bird_2.jpg");
        let watermark = open_image_from_bytes(data).unwrap();

        transform::resize(&watermark, 64, 64, SamplingFilter::Nearest)
    };
}

use crate::{
    Contrast, Crop, Filter, Fliph, Flipv, Resize, Spec, Watermark,
    engine::{Engine, SepcTransform},
    filter, resize, spec,
};

pub struct Photon(PhotonImage);

impl TryFrom<Bytes> for Photon {
    type Error = Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        Ok(Self(open_image_from_bytes(&value.to_vec())?))
    }
}

impl Engine for Photon {
    fn apply(&mut self, specs: &[Spec]) {
        for spec in specs {
            match spec.data {
                Some(spec::Data::Resize(resize)) => self.transform(resize),
                Some(spec::Data::Crop(crop)) => self.transform(crop),
                Some(spec::Data::Flipv(flipv)) => self.transform(flipv),
                Some(spec::Data::Fliph(fliph)) => self.transform(fliph),
                Some(spec::Data::Contrast(contrast)) => self.transform(contrast),
                Some(spec::Data::Filter(filter)) => self.transform(filter),
                Some(spec::Data::Watermark(watermark)) => self.transform(watermark),
                None => {}
            }
        }
    }

    fn process(self, format: ImageFormat) -> Vec<u8> {
        image_to_bytes(self.0.clone(), format)
    }
}

fn image_to_bytes(img: PhotonImage, format: ImageFormat) -> Vec<u8> {
    let rawpixels = img.get_raw_pixels();
    let width = img.get_width();
    let height = img.get_height();
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    let img = ImageBuffer::from_raw(width, height, rawpixels).unwrap();
    DynamicImage::ImageRgba8(img)
        .write_to(&mut cursor, format)
        .unwrap();
    bytes
}

impl SepcTransform<Resize> for Photon {
    fn transform(&mut self, op: Resize) {
        let img = match resize::ResizeType::try_from(op.rtype).unwrap() {
            resize::ResizeType::Normal => transform::resize(
                &mut self.0,
                op.width,
                op.height,
                resize::SampleFilter::try_from(op.filter).unwrap().into(),
            ),
            resize::ResizeType::SeamCarve => {
                transform::seam_carve(&mut self.0, op.width, op.height)
            }
        };
        self.0 = img;
    }
}

impl SepcTransform<Crop> for Photon {
    fn transform(&mut self, op: Crop) {
        let img = transform::crop(&mut self.0, op.x1, op.y1, op.x2, op.y2);
        self.0 = img;
    }
}

impl SepcTransform<Flipv> for Photon {
    fn transform(&mut self, _op: Flipv) {
        transform::flipv(&mut self.0);
    }
}

impl SepcTransform<Fliph> for Photon {
    fn transform(&mut self, _op: Fliph) {
        transform::fliph(&mut self.0);
    }
}

impl SepcTransform<Contrast> for Photon {
    fn transform(&mut self, op: Contrast) {
        adjust_contrast(&mut self.0, op.contrast);
    }
}

impl SepcTransform<Filter> for Photon {
    fn transform(&mut self, op: Filter) {
        match filter::Filter::try_from(op.filter) {
            Ok(filter::Filter::Unspecified) => {}
            Ok(f) => filters::filter(&mut self.0, f.as_str_name()),
            Err(_) => {}
        }
    }
}

impl SepcTransform<Watermark> for Photon {
    fn transform(&mut self, op: Watermark) {
        watermark(&mut self.0, &WATERMARK_FONT, op.x as i64, op.y as i64);
    }
}
