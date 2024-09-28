use std::collections::HashMap;
use image::DynamicImage;
use kiddo::{distance_metric::DistanceMetric, KdTree};
use palette::{color_difference::DeltaE, IntoColor, Lab, Srgb};

pub struct DalImageConverter {
    tree: KdTree<f32, 3>,
    index_map: HashMap<u64, [u8; 3]>,
    dim: (u32, u32),
}

// Convert an RGB color to CIELAB for accurate color comparison
fn rgb_to_lab(rgba: [u8; 3]) -> Lab {
    let srgb = Srgb::new(
        rgba[0] as f32 / 255.0,
        rgba[1] as f32 / 255.0,
        rgba[2] as f32 / 255.0,
    );
    srgb.into_color()
}

// Define a function to compute the CIEDE2000 distance
fn ciede2000_distance(c1: Lab, c2: Lab) -> f32 {
    c1.delta_e(c2)
}

pub struct CiedeDist;
impl DistanceMetric<f32, 3> for CiedeDist {
    fn dist(a: &[f32; 3], b: &[f32; 3]) -> f32 {
        let c1 = Lab::new(a[0], a[1], a[2]);
        let c2 = Lab::new(b[0], b[1], b[2]);
        ciede2000_distance(c1, c2)
    }

    fn dist1(a: f32, b: f32) -> f32 {
        a - b
    }
}

impl DalImageConverter {
    pub fn new(palette: &[[u8; 3]], dim: (u32, u32)) -> Self {
        let mut kd_tree: KdTree<f32, 3> = KdTree::new();
        let mut index_map: HashMap<u64, [u8; 3]> = HashMap::new();
        for (i, &color) in palette.iter().enumerate() {
            let lab_color = rgb_to_lab(color);
            kd_tree.add(&[lab_color.l, lab_color.a, lab_color.b], i as u64);
            index_map.insert(i as u64, color);
        }

        Self {
            tree: kd_tree,
            index_map,
            dim,
        }
    }


    fn get_nearest(&self, rgba: [u8; 3]) -> [u8; 3] {
        let lab = rgb_to_lab(rgba);
        let nearest = self
            .tree
            .nearest_one::<CiedeDist>(&[lab.l, lab.a, lab.b])
            .item;
        *self.index_map.get(&nearest).expect("Color not found")
    }


    pub fn resize_and_rotate(&self, img: DynamicImage, auto_rotate: bool) -> DynamicImage {
        // If width is smaller than height, rotate the image
        let img = if img.width() < img.height() && auto_rotate {
            img.rotate90()
        } else {
            img
        };

        img.resize_exact(self.dim.0, self.dim.1, image::imageops::FilterType::Lanczos3)
    }

    pub fn convert(&self, mut img: image::RgbImage) -> image::RgbImage {
        for px in img.pixels_mut() {
            px.0 = self.get_nearest(px.0);
        }

        img
    }

    pub fn convert_alpha(
        &self,
        mut img: image::RgbaImage,
        trans_color: [u8; 3],
    ) -> image::RgbaImage {
        for px in img.pixels_mut() {
            let c = if px.0[3] == 255 {
                self.get_nearest([px.0[0], px.0[1], px.0[2]])
            } else {
                dbg!(trans_color)
            };

            px.0 = [c[0], c[1], c[2], 255];
        }

        img
    }

    pub fn process(&self, img: DynamicImage, auto_rotate: bool) -> DynamicImage {
        let img = self.resize_and_rotate(img, auto_rotate).to_rgb8();
        dioxus_logger::tracing::info!("resized: {} {}", img.height(), img.width());
        let img = self.convert(img);
        dioxus_logger::tracing::info!("converted: {} {}", img.height(), img.width());
        DynamicImage::ImageRgb8(img)
    }
}

const PALETTE: [[u8; 3]; 18] = [
    [0, 0, 0],
    [0, 0, 0],
    [255, 255, 255],
    [255, 0, 0],
    [255, 124, 123],
    [120, 0, 2],
    [10, 13, 255],
    [125, 134, 255],
    [3, 0, 122],
    [0, 255, 10],
    [150, 255, 154],
    [0, 115, 4],
    [255, 232, 0],
    [255, 245, 140],
    [110, 94, 0],
    [255, 99, 0],
    [255, 179, 131],
    [113, 55, 18],
];
const DIM: (u32, u32) = (87, 60);

impl Default for DalImageConverter {
    fn default() -> Self {
        Self::new(&PALETTE, DIM)
    }
}