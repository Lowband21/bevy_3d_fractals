use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_rapier3d::prelude::*;
use core::f32::consts::PI;

mod flycam;
use crate::flycam::{FlyCam, NoCameraPlayerPlugin};

fn main() {
    App::new()
        .insert_resource(NeedsUpdate(true))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(NoCameraPlayerPlugin)
        .run();
}

/// A marker component for our shapes so we can query them separately from other entities
#[derive(Component)]
struct Shape;

#[derive(Resource)]
struct NeedsUpdate(bool);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a debug material, or use your own
    //let debug_material = materials.add(StandardMaterial {
    //    base_color_texture: Some(images.add(uv_debug_texture())),
    //    ..default()
    //});

    //// Initialize fractal shape (e.g., a cube for demonstration)
    //let fractal_shape = meshes.add(shape::Cube { size: 1.0 }.into());

    //// Spawn fractal entity
    //commands.spawn((
    //    PbrBundle {
    //        mesh: fractal_shape,
    //        material: materials.add(Color::GREEN.into()),
    //        transform: Transform::from_xyz(0.0, 0.0, 0.0)
    //            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
    //        visibility: Visibility::Hidden,
    //        ..default()
    //    },
    //    Shape, // Custom marker component
    //));
    // Create a base tetrahedron mesh
    // Create a new mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // Base vertices of a tetrahedron
    let vertices = [
        [0.0, 0.0, 0.0],                                     // vertex 0
        [1.0, 0.0, 0.0],                                     // vertex 1
        [0.5, 0.0, 3.0f32.sqrt() / 2.0],                     // vertex 2
        [0.5, (6.0f32).sqrt() / 3.0, (3.0f32).sqrt() / 6.0], // vertex 3
    ];

    let indices = [
        0, 1, 2, // triangle 0
        0, 2, 3, // triangle 1
        0, 3, 1, // triangle 2
        1, 3, 2, // triangle 3
    ];

    // Create a mesh from the vertices and triangle indices.
    for vertex in &vertices {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![*vertex]);
    }

    mesh.set_indices(Some(Indices::U32(indices.to_vec())));

    // Add the custom mesh to the resource and spawn it.
    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::GREEN.into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

    // Initialize a light source
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // Initialize camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        FlyCam,
    ));
}
// Recursively create the Sierpinski tetrahedrons
fn create_tetrahedron(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    scale: f32,
    iteration: u32,
    tetrahedron_mesh: &Handle<Mesh>,
) {
    if iteration == 0 {
        return;
    }

    // scale down the tetrahedron for this iteration
    let new_scale = scale / 2.0;

    // For a Sierpinski tetrahedron, we place smaller tetrahedrons at the corners of the current one
    let offsets = [
        Vec3::new(1.0, 0.0, -1.0 / (2.0_f32).sqrt()), // front right
        Vec3::new(-1.0, 0.0, -1.0 / (2.0_f32).sqrt()), // front left
        Vec3::new(0.0, 0.0, 1.0 / (2.0_f32).sqrt()),  // back middle
        Vec3::new(0.0, (2.0_f32).sqrt(), 0.0),        // top
    ];

    for &offset in offsets.iter() {
        let new_position = position + offset * new_scale * 2.0;
        commands.spawn(PbrBundle {
            mesh: tetrahedron_mesh.clone(),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_scale(Vec3::splat(new_scale)) // Scale down
                .with_translation(new_position), // Move to the correct position
            ..Default::default()
        });

        // Recursive call to create the smaller tetrahedrons
        create_tetrahedron(
            commands,
            materials,
            new_position,
            new_scale,
            iteration - 1,
            tetrahedron_mesh,
        );
    }
}
const MAX_ITERATIONS: u32 = 4; // Adjust this for the desired depth.
const SCALING_FACTOR: f32 = 1.0 / 3.0; // Menger Sponge is divided into thirds.

fn generate_fractal(
    position: Vec3, // This is the center of the imaginary larger cube
    scale: f32,     // This is the scale of the individual cubes
    iteration: u32,
    commands: &mut Commands,
    mesh_handle: &Handle<Mesh>,
    material_handle: &Handle<StandardMaterial>,
) {
    if iteration == 0 {
        // At iteration 0, do nothing. We start building from iteration 1.
        return;
    }

    let offset = scale; // The distance from the center to place the smaller cubes

    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                // Skip the center cube and the centers of each face
                if (i == 1 && j == 1) || (i == 1 && k == 1) || (j == 1 && k == 1) {
                    continue;
                }

                let new_position = Vec3::new(
                    position.x + (i as f32 - 1.0) * offset,
                    position.y + (j as f32 - 1.0) * offset,
                    position.z + (k as f32 - 1.0) * offset,
                );

                // Create a single base cube with Menger Sponge texture at the new position.
                commands.spawn(PbrBundle {
                    mesh: mesh_handle.clone(),
                    material: material_handle.clone(),
                    transform: Transform {
                        translation: new_position,
                        scale: Vec3::new(scale, scale, scale),
                        ..Default::default()
                    },
                    ..Default::default()
                });

                if iteration > 1 {
                    // Recursive call to place even larger structures, if needed
                    generate_fractal(
                        new_position,
                        scale,
                        iteration - 1,
                        commands,
                        mesh_handle,
                        material_handle,
                    );
                }
            }
        }
    }
}

fn update(
    mut commands: Commands,
    mut query: Query<(&Handle<Mesh>, &Handle<StandardMaterial>, &Transform, &Shape)>,
    mut needs_update: ResMut<NeedsUpdate>,
) {
    if needs_update.0 {
        for (mesh_handle, material_handle, transform, _) in query.iter_mut() {
            // Start generating the fractal from an initial position, scale, and iteration
            let initial_position = Vec3::new(0.0, 0.0, 0.0);
            let initial_scale = 1.0;
            generate_fractal(
                initial_position,
                initial_scale,
                4,
                &mut commands,
                mesh_handle,
                material_handle,
            );
        }
        needs_update.0 = false;
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}

fn create_shape(
    position: Vec3,
    scale: f32,
    commands: &mut Commands,
    fractal_mesh: &Handle<Mesh>,
    debug_material: &Handle<StandardMaterial>,
) {
    commands.spawn((
        PbrBundle {
            mesh: fractal_mesh.clone(),
            material: debug_material.clone(),
            transform: Transform::from_scale(Vec3::splat(scale))
                .with_translation(position)
                .with_rotation(Quat::from_rotation_x(-PI)),
            ..default()
        },
        Shape, // Custom marker component
    ));
}
