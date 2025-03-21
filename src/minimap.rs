use wgpu::util::DeviceExt;
use image::{ImageBuffer, Rgba};
use crate::texture::Texture;

use glam::Vec3;

// 小地图结构体
pub struct Minimap {
    pub texture: Texture,
    pub size: u32,
    pub scale: f32,
    pub position: [f32; 2], // 屏幕上的位置 (左上角)
    pub dimensions: [f32; 2], // 小地图尺寸
    pub player_marker_color: [u8; 4], // 玩家标记颜色
}

impl Minimap {
    // 创建新的小地图
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        map_data: &Vec<Vec<u8>>,
        size: u32,
        scale: f32,
        position: [f32; 2],
        dimensions: [f32; 2],
    ) -> Self {
        // 创建小地图纹理
        let texture = Self::create_minimap_texture(device, queue, map_data, size);
        
        Self {
            texture,
            size,
            scale,
            position,
            dimensions,
            player_marker_color: [255, 0, 0, 255], // 红色
        }
    }
    
    // 创建小地图纹理
    fn create_minimap_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        map_data: &Vec<Vec<u8>>,
        size: u32,
    ) -> Texture {
        // 创建一个新的图像缓冲区
        let mut img = ImageBuffer::new(size, size);
        
        // 计算地图数据和纹理的比例
        let map_height = map_data.len();
        let map_width = if map_height > 0 { map_data[0].len() } else { 0 };
        
        let scale_x = size as f32 / map_width as f32;
        let scale_y = size as f32 / map_height as f32;
        
        // 填充图像缓冲区
        for (y, row) in map_data.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                let pixel_x = (x as f32 * scale_x) as u32;
                let pixel_y = (y as f32 * scale_y) as u32;
                let pixel_width = (scale_x.ceil()) as u32;
                let pixel_height = (scale_y.ceil()) as u32;
                
                // 根据地图数据设置像素颜色
                let color = match cell {
                    0 => Rgba([200, 200, 200, 255]), // 空地 - 浅灰色
                    1 => Rgba([50, 50, 50, 255]),   // 墙壁 - 深灰色
                    _ => Rgba([0, 0, 0, 0]),        // 其他 - 透明
                };
                
                // 填充像素区域
                for dy in 0..pixel_height {
                    for dx in 0..pixel_width {
                        let px = pixel_x + dx;
                        let py = pixel_y + dy;
                        if px < size && py < size {
                            img.put_pixel(px, py, color);
                        }
                    }
                }
            }
        }
        
        // 将图像转换为RGBA格式并创建纹理
        let rgba = img.into_raw();
        
        let texture_size = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };
        
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("Minimap Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );
        
        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size),
                rows_per_image: Some(size),
            },
            texture_size,
        );
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        
        Texture {
            texture,
            view,
            sampler,
        }
    }
    
    // 更新小地图上的玩家位置
    pub fn update_player_position(
        &self,
        queue: &wgpu::Queue,
        player_position: Vec3,
        map_data: &Vec<Vec<u8>>,
        map_scale: f32, // 地图单位到游戏世界单位的比例
        map_offset: [f32; 2], // 地图原点在游戏世界中的偏移
    ) {
        // 创建一个新的图像缓冲区，复制当前小地图
        let mut img = ImageBuffer::new(self.size, self.size);
        
        // 计算地图数据和纹理的比例
        let map_height = map_data.len();
        let map_width = if map_height > 0 { map_data[0].len() } else { 0 };
        
        let scale_x = self.size as f32 / map_width as f32;
        let scale_y = self.size as f32 / map_height as f32;
        
        // 填充图像缓冲区
        for (y, row) in map_data.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                let pixel_x = (x as f32 * scale_x) as u32;
                let pixel_y = (y as f32 * scale_y) as u32;
                let pixel_width = (scale_x.ceil()) as u32;
                let pixel_height = (scale_y.ceil()) as u32;
                
                // 根据地图数据设置像素颜色
                let color = match cell {
                    0 => Rgba([200, 200, 200, 255]), // 空地 - 浅灰色
                    1 => Rgba([50, 50, 50, 255]),   // 墙壁 - 深灰色
                    _ => Rgba([0, 0, 0, 0]),        // 其他 - 透明
                };
                
                // 填充像素区域
                for dy in 0..pixel_height {
                    for dx in 0..pixel_width {
                        let px = pixel_x + dx;
                        let py = pixel_y + dy;
                        if px < self.size && py < self.size {
                            img.put_pixel(px, py, color);
                        }
                    }
                }
            }
        }
        
        // 计算玩家在小地图上的位置
        let player_map_x = (player_position.x - map_offset[0]) / map_scale;
        let player_map_z = (player_position.z - map_offset[1]) / map_scale;
        
        let player_pixel_x = (player_map_x * scale_x) as u32;
        let player_pixel_y = (player_map_z * scale_y) as u32; // 使用y坐标命名，保持一致性
        
        // 在小地图上绘制玩家标记（红点）
        let marker_size = 5u32; // 增大标记大小，使其更明显
        let marker_half = marker_size / 2;
        
        // Calculate the starting position, ensuring we don't overflow
        let start_x = if player_pixel_x >= marker_half { player_pixel_x - marker_half } else { 0 };
        let start_y = if player_pixel_y >= marker_half { player_pixel_y - marker_half } else { 0 };
        
        // Calculate the ending position, ensuring we don't exceed the texture size
        let end_x = (player_pixel_x + marker_half).min(self.size);
        let end_y = (player_pixel_y + marker_half).min(self.size);
        
        // Draw the marker
        for py in start_y..end_y {
            for px in start_x..end_x {
                img.put_pixel(px, py, Rgba(self.player_marker_color));
            }
        }
        
        // 将更新后的图像写入纹理
        let rgba = img.into_raw();
        
        let texture_size = wgpu::Extent3d {
            width: self.size,
            height: self.size,
            depth_or_array_layers: 1,
        };
        
        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.size),
                rows_per_image: Some(self.size),
            },
            texture_size,
        );
    }
    
    // 创建小地图的顶点和索引缓冲区 - 为2D UI渲染优化
    pub fn create_vertices_and_indices(
        &self,
        device: &wgpu::Device,
        screen_width: u32,
        screen_height: u32,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        // 计算小地图在屏幕上的位置和大小
        let x = self.position[0];
        let y = self.position[1];
        let width = self.dimensions[0];
        let height = self.dimensions[1];
        
        // 将屏幕坐标转换为标准化设备坐标 (-1 到 1)
        let left = 2.0 * x / screen_width as f32 - 1.0;
        let right = 2.0 * (x + width) / screen_width as f32 - 1.0;
        let top = 1.0 - 2.0 * y / screen_height as f32;
        let bottom = 1.0 - 2.0 * (y + height) / screen_height as f32;
        
        // 创建顶点数据 - 简化为仅包含位置和纹理坐标
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        struct Vertex {
            position: [f32; 3],
            tex_coords: [f32; 2],
        }

        unsafe impl bytemuck::Pod for Vertex {}
        unsafe impl bytemuck::Zeroable for Vertex {}
        
        let vertices = [
            Vertex { position: [left, top, 0.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [right, top, 0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [right, bottom, 0.0], tex_coords: [1.0, 1.0] },
            Vertex { position: [left, bottom, 0.0], tex_coords: [0.0, 1.0] },
        ];
        
        let indices = [0u16, 1, 2, 0, 2, 3];
        
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Minimap Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Minimap Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        
        (vertex_buffer, index_buffer, indices.len() as u32)
    }
    
    // 返回顶点缓冲区布局描述 - 为UI着色器提供
    pub fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress, // 3个位置 + 2个纹理坐标
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}