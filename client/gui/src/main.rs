use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, setup);
    app.add_systems(Update, rotation_systeme);
    app.run();
}

#[derive(Component)]
struct MonCarre;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            ..default()
        },
        MonCarre, // add the type to the entity
    ));
}

// Système de mise à jour (la logique du jeu)
fn rotation_systeme(time: Res<Time>, mut requests: Query<&mut Transform, With<MonCarre>>) {
    // On boucle sur tout ce qui possède l'étiquette "MonCarre"
    for mut transform in &mut requests {
        // Fait tourner l'objet sur l'axe Z
        transform.rotate_z(2.0 * time.delta_seconds());
    }
}
