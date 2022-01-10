#[cfg(feature = "3d-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "3d-io")))]
mod threed;
#[doc(inline)]
#[cfg(feature = "3d-io")]
pub use threed::*;

#[cfg(feature = "obj-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "obj-io")))]
mod obj;
#[doc(inline)]
#[cfg(feature = "obj-io")]
pub use obj::*;

#[cfg(feature = "gltf-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "gltf-io")))]
mod gltf;
#[doc(inline)]
#[cfg(feature = "gltf-io")]
pub use self::gltf::*;

#[cfg(feature = "image-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "image-io")))]
mod img;
#[cfg(feature = "image-io")]
#[doc(inline)]
pub use img::*;
