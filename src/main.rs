//! Reducing rapier3d motor confusion to a simple case....

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            spacebar.run_if(resource_changed::<Input<KeyCode>>()),
        )
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // start with physics off
    rapier_config.physics_pipeline_active = false;

    // lights!
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

    // camera!
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.5, 4.0).looking_at(Vec3::new(0., 0.5, 0.5), Vec3::Y),
        ..default()
    });

    // plane
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(5.0).into()),
            material: materials.add(Color::SEA_GREEN.into()),
            ..default()
        })
        .insert(RigidBody::Fixed)
        .with_children(|p| {
            p.spawn(Collider::cuboid(2.5, 0.25, 2.5))
                .insert(TransformBundle::from(Transform::from_xyz(0.0, -0.25, 0.0)));
        });

    // a cube that will be the "body" of our clock.
    let clock = commands
        .spawn(PbrBundle {
            mesh: meshes.add(
                shape::Box {
                    min_x: -0.5,
                    max_x: 0.5,
                    min_y: 0.0,
                    max_y: 1.0,
                    min_z: -0.5,
                    max_z: 0.5,
                }
                .into(),
            ),
            //material: materials.add(Color::CYAN.into()),
            material: materials.add(
                Color::Rgba {
                    red: 0.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 0.95,
                }
                .into(),
            ),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .with_children(|p| {
            p.spawn(Collider::cuboid(0.5, 0.5, 0.5))
                .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.5, 0.0)));
        })
        .id();

    // hand of the clock.
    let hand = commands
        .spawn(PbrBundle {
            mesh: meshes.add(
                shape::Box {
                    min_x: -1. / 16.,
                    max_x: 1. / 16.,
                    min_y: 0.5 + 1. / 32.,
                    max_y: 1. - 1. / 32.,
                    min_z: 0.5,
                    max_z: 0.5 + 1. / 16.,
                }
                .into(),
            ),
            material: materials.add(Color::ANTIQUE_WHITE.into()),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .with_children(|p| {
            p.spawn(Collider::cuboid(1. / 16., 0.25 - 1. / 32., 1. / 32.))
                .insert(TransformBundle::from(Transform::from_xyz(
                    0.0,
                    0.75,
                    0.5 + 1. / 32.,
                )));
        })
        .id();

    // since the two entities are defined from the origin (0,0,0) and haven't moved,
    // connection point is the same for both, which makes this _much_ simpler.
    let motor = RevoluteJointBuilder::new(Vec3::Z)
        .local_anchor1(Vec3::new(0.0, 0.5, 0.5))
        .local_anchor2(Vec3::new(0.0, 0.5, 0.5));

    // To my surprise, order only seems to matter in which direction the joint is facing,
    // which determines whether "clockwise" is positive or negative.
    commands
        .entity(clock)
        .insert(ImpulseJoint::new(hand, motor));

    info!("Press space to start the physics engine, and then again to advance the clock.")
}

fn spacebar(
    keys: Res<Input<KeyCode>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut query: Query<&mut ImpulseJoint>,
    mut hour: Local<f32>,
) {
    // spacebar is the only command.
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    // if physics is off, turn it on.
    if !rapier_config.physics_pipeline_active {
        rapier_config.physics_pipeline_active = true;
        info!("Physics engine now active.");
        return;
    }

    let mut impulsejoint = query.get_single_mut().unwrap();
    let motor = impulsejoint.data.as_revolute_mut().unwrap();

    *hour = (*hour + 1.0) % 12.;
    info!("Hour set to {}", *hour);

    // This only works up to 6, and then goes backwards (7-11 roughly become 5-1)
    // It also only works with very very high stiffness. Less than this and the
    // hand just spins infinitely.
    motor.set_motor_position(PI / 12. * *hour, Real::MAX, 0.0);
}
