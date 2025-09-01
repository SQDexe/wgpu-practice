use {
    image::{
        load_from_memory,
        DynamicImage,
        GenericImageView
        },
    anyhow::Result as DynResult,
    wgpu::{
        *,
        Texture as WGPUTexture
        }
    };

pub struct Texture {
    texture: WGPUTexture,
    view: TextureView,
    sampler: Sampler
    }

impl Texture {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8], label: Option<&str>) -> DynResult<Self> {
        Self::from_image(
            device,
            queue,
            load_from_memory(bytes)?,
            label
            )
        }

    pub fn from_image(device: &Device, queue: &Queue, img: DynamicImage, label: Option<&str>) -> DynResult<Self> {
        let (width, height) = img.dimensions();

        let size = Extent3d {
            width,
            height,
            // All textures are stored as 3D, we represent our 2D texture  by setting depth to 1
            depth_or_array_layers: 1
            };

        let texture = device.create_texture(&TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            // Most images are stored with sRGB format
            format: TextureFormat::Rgba8UnormSrgb,
            // TEXTURE_BINDING - for usage in shaders, COPY_DST - for coping data into
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[]
            });

        queue.write_texture(
            TexelCopyTextureInfo {
                aspect: TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO
                },
            &img.to_rgba8(),
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height)
                },
            size
            );

        // Allow WGPU to define view by itself
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            .. Default::default()
            });

        Ok(Self { texture, view, sampler })
        }

    pub fn create_depth_texture(device: &Device, config: &SurfaceConfiguration, label: Option<&str>) -> Self {
        let size = Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1
            };

        let texture = device.create_texture(&TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
            });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            .. Default::default()
            });

        Self { texture, view, sampler }
        }

    pub const fn get_sampler(&self) -> &Sampler {
        &self.sampler
        }

    pub const fn get_view(&self) -> &TextureView {
        &self.view
        }
    }