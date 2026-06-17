use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorSpace {
    SRGB,
    AdobeRgb,
    ACES2065,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorSpaceKind {
    SRGB,
    AdobeRgb,
    ACES2065,
    Linear,
    GenericGamma,
}

pub fn color_space_from_kind(kind: ColorSpaceKind) -> ColorSpace {
    match kind {
        ColorSpaceKind::SRGB => ColorSpace::SRGB,
        ColorSpaceKind::AdobeRgb => ColorSpace::AdobeRgb,
        ColorSpaceKind::ACES2065 => ColorSpace::ACES2065,
        ColorSpaceKind::Linear => ColorSpace::Linear,
        ColorSpaceKind::GenericGamma => ColorSpace::SRGB,
    }
}

impl From<ColorSpace> for ColorSpaceKind {
    fn from(space: ColorSpace) -> Self {
        match space {
            ColorSpace::SRGB => ColorSpaceKind::SRGB,
            ColorSpace::AdobeRgb => ColorSpaceKind::AdobeRgb,
            ColorSpace::ACES2065 => ColorSpaceKind::ACES2065,
            ColorSpace::Linear => ColorSpaceKind::Linear,
        }
    }
}

pub trait ColorSpaceConverter: Send + Sync + std::fmt::Debug {
    fn convert(&self, source: [f32; 3], source_space: ColorSpace, target_space: ColorSpace) -> [f32; 3];
    fn get_gamut_info(&self) -> GamutInfo;
}

#[derive(Debug)]
pub struct GamutInfo {
    pub is_hdr: bool,
    pub primaries: GamutPrimaries,
}

#[derive(Debug)]
pub struct GamutPrimaries {
    pub r: (f32, f32),
    pub g: (f32, f32),
    pub b: (f32, f32),
    pub white: (f32, f32),
}

#[derive(Debug)]
pub struct ColorSpaceConverterImpl;

impl ColorSpaceConverterImpl {
    fn srgb_gamma(c: f32) -> f32 {
        if c <= 0.0031308 { 12.92 * c } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
    }

    fn srgb_inv_gamma(c: f32) -> f32 {
        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    }

    fn apply_matrix(m: &[[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
        [
            m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
            m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
            m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
        ]
    }

    fn to_xyz(linear_rgb: [f32; 3], space: ColorSpace) -> [f32; 3] {
        match space {
            ColorSpace::SRGB | ColorSpace::Linear => {
                Self::apply_matrix(&SRGB_TO_XYZ, linear_rgb)
            }
            ColorSpace::AdobeRgb => {
                Self::apply_matrix(&ADOBE_TO_XYZ, linear_rgb)
            }
            ColorSpace::ACES2065 => {
                Self::apply_matrix(&ACES_TO_XYZ, linear_rgb)
            }
        }
    }

    fn from_xyz(xyz: [f32; 3], space: ColorSpace) -> [f32; 3] {
        match space {
            ColorSpace::SRGB | ColorSpace::Linear => {
                Self::apply_matrix(&XYZ_TO_SRGB, xyz)
            }
            ColorSpace::AdobeRgb => {
                Self::apply_matrix(&XYZ_TO_ADOBE, xyz)
            }
            ColorSpace::ACES2065 => {
                Self::apply_matrix(&XYZ_TO_ACES, xyz)
            }
        }
    }

    fn to_linear(rgb: [f32; 3], space: ColorSpace) -> [f32; 3] {
        match space {
            ColorSpace::SRGB => [Self::srgb_inv_gamma(rgb[0]), Self::srgb_inv_gamma(rgb[1]), Self::srgb_inv_gamma(rgb[2])],
            ColorSpace::AdobeRgb => [rgb[0].powf(2.2), rgb[1].powf(2.2), rgb[2].powf(2.2)],
            ColorSpace::ACES2065 | ColorSpace::Linear => rgb,
        }
    }

    fn from_linear(rgb: [f32; 3], space: ColorSpace) -> [f32; 3] {
        match space {
            ColorSpace::SRGB => [Self::srgb_gamma(rgb[0]), Self::srgb_gamma(rgb[1]), Self::srgb_gamma(rgb[2])],
            ColorSpace::AdobeRgb => [rgb[0].powf(1.0 / 2.2), rgb[1].powf(1.0 / 2.2), rgb[2].powf(1.0 / 2.2)],
            ColorSpace::ACES2065 | ColorSpace::Linear => rgb,
        }
    }
}

impl ColorSpaceConverter for ColorSpaceConverterImpl {
    fn convert(&self, source: [f32; 3], source_space: ColorSpace, target_space: ColorSpace) -> [f32; 3] {
        if source_space == target_space {
            return source;
        }
        let linear = Self::to_linear(source, source_space);
        let xyz = Self::to_xyz(linear, source_space);
        let target_linear = Self::from_xyz(xyz, target_space);
        Self::from_linear(target_linear, target_space)
    }

    fn get_gamut_info(&self) -> GamutInfo {
        GamutInfo { is_hdr: false, primaries: GamutPrimaries {
            r: (0.64, 0.33),
            g: (0.30, 0.60),
            b: (0.15, 0.06),
            white: (0.3127, 0.3290),
        }}
    }
}

// sRGB to XYZ (D65) — from IEC 61966-2-1
static SRGB_TO_XYZ: [[f32; 3]; 3] = [
    [0.4124564, 0.3575761, 0.1804375],
    [0.2126729, 0.7151522, 0.0721750],
    [0.0193339, 0.119_192, 0.9503041],
];
static XYZ_TO_SRGB: [[f32; 3]; 3] = [
    [ 3.2404542, -1.5371385, -0.4985314],
    [-0.969_266,  1.8760108,  0.0415560],
    [ 0.0556434, -0.2040259,  1.0572252],
];

// Adobe RGB (1998) to XYZ (D65) — from Adobe spec
static ADOBE_TO_XYZ: [[f32; 3]; 3] = [
    [0.5767309, 0.185_554, 0.1881852],
    [0.2973769, 0.6273491, 0.0752741],
    [0.0270343, 0.0706872, 0.9911085],
];
static XYZ_TO_ADOBE: [[f32; 3]; 3] = [
    [ 2.041_369, -0.5649464, -0.3446944],
    [-0.969_266,  1.8760108,  0.0415560],
    [ 0.0134474, -0.1183897,  1.0154096],
];

// ACES2065-1 (AP0) to XYZ (D60) — from SMPTE ST 2065-1
static ACES_TO_XYZ: [[f32; 3]; 3] = [
    [ 0.9525524,  0.0000000,  0.0000937],
    [ 0.3439664,  0.7281661, -0.0721325],
    [ 0.0000000,  0.0000000,  1.0088252],
];
static XYZ_TO_ACES: [[f32; 3]; 3] = [
    [ 1.049_811,  0.0000000, -0.0000975],
    [-0.495_903,  1.373_313,  0.0982401],
    [ 0.0000000,  0.0000000,  0.991_252],
];
