mod abi;

pub use abi::*;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use photon_rs::transform::SamplingFilter;
use prost::Message;

impl ImageSpec {
    pub fn new(specs: Vec<Spec>) -> Self {
        Self { specs }
    }
}

impl From<&ImageSpec> for String {
    fn from(spec: &ImageSpec) -> Self {
        let data = spec.encode_to_vec();
        BASE64_URL_SAFE_NO_PAD.encode(data)
    }
}

impl TryFrom<&str> for ImageSpec {
    type Error = anyhow::Error;

    fn try_from(spec: &str) -> Result<Self, Self::Error> {
        let data = BASE64_URL_SAFE_NO_PAD.decode(spec)?;
        Ok(ImageSpec::decode(&data[..])?)
    }
}

impl From<resize::SampleFilter> for SamplingFilter {
    fn from(value: resize::SampleFilter) -> Self {
        match value {
            resize::SampleFilter::Undefined => SamplingFilter::Nearest,
            resize::SampleFilter::Nearest => SamplingFilter::Nearest,
            resize::SampleFilter::Triangle => SamplingFilter::Triangle,
            resize::SampleFilter::CatmullRom => SamplingFilter::CatmullRom,
            resize::SampleFilter::Gaussian => SamplingFilter::Gaussian,
            resize::SampleFilter::Lanczos3 => SamplingFilter::Lanczos3,
        }
    }
}

impl Spec {
    pub fn resize(
        width: u32,
        height: u32,
        rtype: resize::ResizeType,
        filter: resize::SampleFilter,
    ) -> Self {
        match rtype {
            resize::ResizeType::Normal => Spec {
                data: Some(spec::Data::Resize(Resize {
                    width,
                    height,
                    rtype: resize::ResizeType::Normal as i32,
                    filter: filter.into(),
                })),
            },
            resize::ResizeType::SeamCarve => Spec {
                data: Some(spec::Data::Resize(Resize {
                    width,
                    height,
                    rtype: resize::ResizeType::SeamCarve as i32,
                    filter: filter.into(),
                })),
            },
        }
    }

    pub fn crop(x1: u32, y1: u32, x2: u32, y2: u32) -> Self {
        Spec {
            data: Some(spec::Data::Crop(Crop { x1, y1, x2, y2 })),
        }
    }

    pub fn flip_h() -> Self {
        Spec {
            data: Some(spec::Data::Fliph(Fliph {})),
        }
    }

    pub fn flip_v() -> Self {
        Spec {
            data: Some(spec::Data::Flipv(Flipv {})),
        }
    }

    pub fn contrast(contrast: f32) -> Self {
        Spec {
            data: Some(spec::Data::Contrast(Contrast { contrast })),
        }
    }

    pub fn filter(filter: filter::Filter) -> Self {
        Spec {
            data: Some(spec::Data::Filter(Filter {
                filter: filter.into(),
            })),
        }
    }

    pub fn watermark(x: u64, y: u64) -> Self {
        Spec {
            data: Some(spec::Data::Watermark(Watermark { x, y })),
        }
    }
}

pub fn print_test_url(url: &str) {
    let spec1 = Spec::resize(
        500,
        800,
        resize::ResizeType::Normal,
        resize::SampleFilter::CatmullRom,
    );
    let spec2 = Spec::watermark(20, 20);
    let spec3 = Spec::filter(filter::Filter::Marine);

    let image_spec = ImageSpec::new(vec![spec1, spec2, spec3]);

    let image_spec_str: String = (&image_spec).into();
    let url = percent_encoding::percent_encode(url.as_bytes(), percent_encoding::NON_ALPHANUMERIC);
    println!("http://localhost:8080/image/{}/{}", image_spec_str, url);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_spec() {
        let spec1 = Spec::resize(
            100,
            100,
            resize::ResizeType::Normal,
            resize::SampleFilter::Nearest,
        );
        let spec2 = Spec::crop(10, 10, 100, 100);
        let spec3 = Spec::flip_h();
        let spec4 = Spec::flip_v();
        let spec5 = Spec::contrast(0.5);
        let spec6 = Spec::filter(filter::Filter::Oceanic);
        let spec7 = Spec::watermark(10, 10);

        let image_spec = ImageSpec::new(vec![spec1, spec2, spec3, spec4, spec5, spec6, spec7]);

        let image_spec_str: String = (&image_spec).into();

        assert_eq!(image_spec, image_spec_str.as_str().try_into().unwrap());
    }
}
