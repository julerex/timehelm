//! Physics simulation module using Rapier3D.
//!
//! Manages physics bodies, collisions, and entity dynamics.
//! Units: centimeters (1 unit = 1 cm).

use rand::Rng;
use rapier3d::prelude::*;
use std::collections::HashMap;

/// Physics simulation world.
///
/// Manages rigid bodies, colliders, and physics simulation.
/// Handles both dynamic objects (balls) and kinematic objects (humans).
pub struct PhysicsWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhaseBvh,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub gravity: Vector<Real>,
    pub integration_parameters: IntegrationParameters,
    pub entity_handles: HashMap<String, RigidBodyHandle>,
}

impl PhysicsWorld {
    /// Create a new physics world with ground plane.
    ///
    /// Initializes physics simulation components and creates a ground collider.
    /// Ground is a large plane (10000x10000 units) with perfect elasticity.
    pub fn new() -> Self {
        let rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        // Create ground plane (large cuboid to match visual ground size of 10000 units)
        // Ground surface is at y=0, so the top of the cuboid is at y=0
        // Using half-extents: 5000 (half of 10000) for x/z, 0.1 for y (thin ground)
        let ground_collider = ColliderBuilder::cuboid(5000.0, 0.1, 5000.0)
            .translation(vector![0.0, -0.1, 0.0]) // Position so top surface is at y=0
            .friction(0.0)
            .restitution(1.0) // Perfect elasticity
            .build();
        collider_set.insert(ground_collider);

        // Create boundary walls around the perimeter to contain bouncing balls
        // Ground is 10000x10000 units, so boundaries are at ±5000
        // Walls are 2000 units tall (20 meters) to contain high bounces
        let wall_height = 2000.0;
        let wall_thickness = 100.0; // 1 meter thick walls
        let ground_half_size = 5000.0;

        // East wall (positive X) - inner edge at x = ground_half_size
        let east_wall = ColliderBuilder::cuboid(wall_thickness, wall_height, ground_half_size)
            .translation(vector![
                ground_half_size + wall_thickness / 2.0,
                wall_height,
                0.0
            ])
            .friction(0.0)
            .restitution(1.0) // Perfect elasticity
            .build();
        collider_set.insert(east_wall);

        // West wall (negative X) - inner edge at x = -ground_half_size
        let west_wall = ColliderBuilder::cuboid(wall_thickness, wall_height, ground_half_size)
            .translation(vector![
                -ground_half_size - wall_thickness / 2.0,
                wall_height,
                0.0
            ])
            .friction(0.0)
            .restitution(1.0) // Perfect elasticity
            .build();
        collider_set.insert(west_wall);

        // North wall (positive Z) - inner edge at z = ground_half_size
        let north_wall = ColliderBuilder::cuboid(ground_half_size, wall_height, wall_thickness)
            .translation(vector![
                0.0,
                wall_height,
                ground_half_size + wall_thickness / 2.0
            ])
            .friction(0.0)
            .restitution(1.0) // Perfect elasticity
            .build();
        collider_set.insert(north_wall);

        // South wall (negative Z) - inner edge at z = -ground_half_size
        let south_wall = ColliderBuilder::cuboid(ground_half_size, wall_height, wall_thickness)
            .translation(vector![
                0.0,
                wall_height,
                -ground_half_size - wall_thickness / 2.0
            ])
            .friction(0.0)
            .restitution(1.0) // Perfect elasticity
            .build();
        collider_set.insert(south_wall);

        Self {
            rigid_body_set,
            collider_set,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            // Earth gravity scaled for 60x time scale (1 real second = 60 game seconds)
            // Base gravity: 9.81 m/s² = 981 cm/s²
            // Scaled by 60² = 3600 (since acceleration = distance/time²)
            gravity: vector![0.0, -981.0 * 3600.0, 0.0], // 3,531,600 cm/s² in game time
            integration_parameters: IntegrationParameters::default(),
            entity_handles: HashMap::new(),
        }
    }

    /// Create a bouncy ball entity with physics simulation.
    ///
    /// Creates a dynamic rigid body (sphere) that responds to gravity and collisions.
    /// Ball has perfect elasticity (restitution = 1.0) and no friction.
    ///
    /// # Arguments
    /// * `entity_id` - Unique identifier for the entity
    /// * `x` - Initial X position (centimeters)
    /// * `z` - Initial Z position (centimeters)
    ///
    /// # Returns
    /// Rigid body handle for physics updates
    pub fn create_bouncy_ball(&mut self, entity_id: String, x: f32, z: f32) -> RigidBodyHandle {
        use rand::Rng;

        // Random initial velocity for trajectory variation
        // Scaled by 60 for 60x time scale (1 real second = 60 game seconds)
        let mut rng = rand::thread_rng();
        let vel_x = rng.gen_range(-100.0..100.0) * 60.0;
        let vel_z = rng.gen_range(-100.0..100.0) * 60.0;
        let vel_y = 0.0; // Zero vertical velocity

        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x, 500.0, z]) // Start at 5 meters (more visible than 10m)
            .linvel(vector![vel_x, vel_y, vel_z])
            .build();
        let handle = self.rigid_body_set.insert(rigid_body);

        // Create sphere collider with perfect elasticity
        // Ball radius is 50 units (50cm = 0.5m) to match visual representation
        // Rapier interprets units as-is, so radius 50 = 50 units (cm in our system)
        // Volume in m³: (4/3)π(0.5)³ ≈ 0.523599 m³
        // For 100g (0.1 kg) mass: density = 0.1 kg / 0.523599 m³ ≈ 0.191 kg/m³
        // However, since Rapier calculates volume from radius directly, and we're using cm,
        // we need density = mass(kg) / volume(m³) where volume is calculated from radius in meters
        // If radius = 50 cm = 0.5 m, then density = 0.1 / ((4/3)π(0.5)³) = 0.191 kg/m³
        let collider = ColliderBuilder::ball(50.0)
            .restitution(1.0) // Perfect elasticity
            .friction(0.0)
            .density(0.191) // 100g ball: 0.1 kg / 0.523599 m³ ≈ 0.191 kg/m³
            .build();
        self.collider_set
            .insert_with_parent(collider, handle, &mut self.rigid_body_set);

        self.entity_handles.insert(entity_id, handle);
        handle
    }

    /// Create a human entity with kinematic physics body.
    ///
    /// Creates a kinematic rigid body (position-controlled, not physics-controlled).
    /// Uses a capsule collider for human shape.
    ///
    /// # Arguments
    /// * `entity_id` - Unique identifier for the entity
    /// * `x` - Initial X position (centimeters)
    /// * `y` - Initial Y position (centimeters)
    /// * `z` - Initial Z position (centimeters)
    ///
    /// # Returns
    /// Rigid body handle for position updates
    pub fn create_human(&mut self, entity_id: String, x: f32, y: f32, z: f32) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::kinematic_position_based()
            .translation(vector![x, y, z])
            .build();
        let handle = self.rigid_body_set.insert(rigid_body);

        // Create capsule collider for human
        let collider = ColliderBuilder::capsule_y(0.5, 0.3)
            .friction(0.5)
            .restitution(0.0)
            .build();
        self.collider_set
            .insert_with_parent(collider, handle, &mut self.rigid_body_set);

        self.entity_handles.insert(entity_id, handle);
        handle
    }

    /// Update human position (for kinematic bodies).
    ///
    /// Sets the position of a kinematic human body directly.
    /// Used when player position is updated from network input.
    ///
    /// # Arguments
    /// * `entity_id` - Entity identifier
    /// * `x` - New X position (centimeters)
    /// * `y` - New Y position (centimeters)
    /// * `z` - New Z position (centimeters)
    pub fn update_human_position(&mut self, entity_id: &str, x: f32, y: f32, z: f32) {
        if let Some(handle) = self.entity_handles.get(entity_id) {
            if let Some(body) = self.rigid_body_set.get_mut(*handle) {
                body.set_translation(vector![x, y, z], true);
            }
        }
    }

    /// Step the physics simulation forward by one time step.
    ///
    /// Updates all physics bodies, handles collisions, and applies gravity.
    /// Also adds random velocity perturbations to balls for visual variety.
    ///
    /// # Arguments
    /// * `_dt` - Delta time in seconds (unused, but kept for API consistency)
    pub fn step(&mut self, _dt: f64) {
        // Add randomness to ball velocities on each step (simulates random bounce effects)
        let mut rng = rand::thread_rng();
        for (entity_id, handle) in &self.entity_handles {
            if entity_id.starts_with("ball_") {
                if let Some(body) = self.rigid_body_set.get_mut(*handle) {
                    let mut linvel = *body.linvel();
                    // Random perturbation to velocity for trajectory variation
                    // Scaled by 60 for 60x time scale
                    if linvel.y < 6.0 && linvel.y > -6.0 {
                        // Near ground, add random horizontal component to maintain speed
                        linvel.x += rng.gen_range(-20.0..20.0) * 60.0;
                        linvel.z += rng.gen_range(-20.0..20.0) * 60.0;
                        body.set_linvel(linvel, true);
                    }
                }
            }
        }

        let hooks: &dyn rapier3d::pipeline::PhysicsHooks = &();
        let events: &dyn rapier3d::pipeline::EventHandler = &();
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            hooks,
            events,
        );
    }

    /// Get entity position from physics world.
    ///
    /// # Arguments
    /// * `entity_id` - Entity identifier
    ///
    /// # Returns
    /// Position tuple (x, y, z) in centimeters, or None if entity not found
    pub fn get_entity_position(&self, entity_id: &str) -> Option<(f32, f32, f32)> {
        let handle = self.entity_handles.get(entity_id)?;
        let body = self.rigid_body_set.get(*handle)?;
        let translation = body.translation();
        Some((translation.x, translation.y, translation.z))
    }

    /// Get entity rotation as Euler angles.
    ///
    /// # Arguments
    /// * `entity_id` - Entity identifier
    ///
    /// # Returns
    /// Rotation tuple (x, y, z) in radians, or None if entity not found
    pub fn get_entity_rotation(&self, entity_id: &str) -> Option<(f32, f32, f32)> {
        let handle = self.entity_handles.get(entity_id)?;
        let body = self.rigid_body_set.get(*handle)?;
        let rotation = body.rotation();
        let euler = rotation.euler_angles();
        Some((euler.0, euler.1, euler.2))
    }

    /// Remove an entity from the physics world.
    ///
    /// Removes the rigid body and all associated colliders.
    ///
    /// # Arguments
    /// * `entity_id` - Entity identifier to remove
    pub fn remove_entity(&mut self, entity_id: &str) {
        if let Some(handle) = self.entity_handles.remove(entity_id) {
            // Remove the rigid body (this will also remove associated colliders)
            self.rigid_body_set.remove(
                handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true,
            );
        }
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}
