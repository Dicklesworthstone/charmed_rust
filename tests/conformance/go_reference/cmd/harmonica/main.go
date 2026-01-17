// Harmonica capture program - captures spring and projectile physics behaviors
package main

import (
	"charmed_conformance/internal/capture"
	"flag"
	"fmt"
	"os"

	"github.com/charmbracelet/harmonica"
)

func main() {
	outputDir := flag.String("output", "output", "Output directory for fixtures")
	flag.Parse()

	fixtures := capture.NewFixtureSet("harmonica", "0.2.0")

	// Capture spring physics behaviors
	captureSpringTests(fixtures)

	// Capture projectile physics behaviors
	captureProjectileTests(fixtures)

	// Capture FPS utility
	captureFPSTests(fixtures)

	if err := fixtures.WriteToFile(*outputDir); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func captureSpringTests(fs *capture.FixtureSet) {
	// Test 1: Default spring from rest to target
	{
		spring := harmonica.NewSpring(harmonica.FPS(60), 6.0, 1.0)
		pos, vel := spring.Update(0.0, 1.0, 1.0/60.0)
		fs.AddTestWithCategory("spring_default_step", "unit",
			capture.SpringInput{
				Frequency: 6.0,
				Damping:   1.0,
				CurrentPos: 0.0,
				TargetPos:  1.0,
				Velocity:   0.0,
				DeltaTime:  1.0/60.0,
			},
			capture.SpringOutput{
				NewPos:      pos,
				NewVelocity: vel,
			},
		)
	}

	// Test 2: Spring already at target
	{
		spring := harmonica.NewSpring(harmonica.FPS(60), 6.0, 1.0)
		pos, vel := spring.Update(1.0, 1.0, 1.0/60.0)
		fs.AddTestWithCategory("spring_at_target", "unit",
			capture.SpringInput{
				Frequency: 6.0,
				Damping:   1.0,
				CurrentPos: 1.0,
				TargetPos:  1.0,
				Velocity:   0.0,
				DeltaTime:  1.0/60.0,
			},
			capture.SpringOutput{
				NewPos:      pos,
				NewVelocity: vel,
			},
		)
	}

	// Test 3: Spring with initial velocity
	{
		spring := harmonica.NewSpring(harmonica.FPS(60), 6.0, 1.0)
		spring.SetVelocity(5.0)
		pos, vel := spring.Update(0.0, 1.0, 1.0/60.0)
		fs.AddTestWithCategory("spring_with_velocity", "unit",
			capture.SpringInput{
				Frequency: 6.0,
				Damping:   1.0,
				CurrentPos: 0.0,
				TargetPos:  1.0,
				Velocity:   5.0,
				DeltaTime:  1.0/60.0,
			},
			capture.SpringOutput{
				NewPos:      pos,
				NewVelocity: vel,
			},
		)
	}

	// Test 4: Under-damped spring (oscillatory)
	{
		spring := harmonica.NewSpring(harmonica.FPS(60), 6.0, 0.3)
		pos, vel := spring.Update(0.0, 1.0, 1.0/60.0)
		fs.AddTestWithNotes("spring_underdamped",
			capture.SpringInput{
				Frequency: 6.0,
				Damping:   0.3,
				CurrentPos: 0.0,
				TargetPos:  1.0,
				Velocity:   0.0,
				DeltaTime:  1.0/60.0,
			},
			capture.SpringOutput{
				NewPos:      pos,
				NewVelocity: vel,
			},
			"Under-damped spring will oscillate around target",
		)
	}

	// Test 5: Over-damped spring (sluggish)
	{
		spring := harmonica.NewSpring(harmonica.FPS(60), 6.0, 2.0)
		pos, vel := spring.Update(0.0, 1.0, 1.0/60.0)
		fs.AddTestWithNotes("spring_overdamped",
			capture.SpringInput{
				Frequency: 6.0,
				Damping:   2.0,
				CurrentPos: 0.0,
				TargetPos:  1.0,
				Velocity:   0.0,
				DeltaTime:  1.0/60.0,
			},
			capture.SpringOutput{
				NewPos:      pos,
				NewVelocity: vel,
			},
			"Over-damped spring approaches target slowly without oscillation",
		)
	}

	// Test 6: Critically damped spring
	{
		spring := harmonica.NewSpring(harmonica.FPS(60), 6.0, 1.0)
		pos, vel := spring.Update(0.0, 1.0, 1.0/60.0)
		fs.AddTestWithNotes("spring_critically_damped",
			capture.SpringInput{
				Frequency: 6.0,
				Damping:   1.0,
				CurrentPos: 0.0,
				TargetPos:  1.0,
				Velocity:   0.0,
				DeltaTime:  1.0/60.0,
			},
			capture.SpringOutput{
				NewPos:      pos,
				NewVelocity: vel,
			},
			"Critically damped spring reaches target fastest without overshoot",
		)
	}

	// Test 7: High frequency spring (snappy)
	{
		spring := harmonica.NewSpring(harmonica.FPS(60), 15.0, 1.0)
		pos, vel := spring.Update(0.0, 1.0, 1.0/60.0)
		fs.AddTestWithCategory("spring_high_frequency", "unit",
			capture.SpringInput{
				Frequency: 15.0,
				Damping:   1.0,
				CurrentPos: 0.0,
				TargetPos:  1.0,
				Velocity:   0.0,
				DeltaTime:  1.0/60.0,
			},
			capture.SpringOutput{
				NewPos:      pos,
				NewVelocity: vel,
			},
		)
	}

	// Test 8: Low frequency spring (slow)
	{
		spring := harmonica.NewSpring(harmonica.FPS(60), 2.0, 1.0)
		pos, vel := spring.Update(0.0, 1.0, 1.0/60.0)
		fs.AddTestWithCategory("spring_low_frequency", "unit",
			capture.SpringInput{
				Frequency: 2.0,
				Damping:   1.0,
				CurrentPos: 0.0,
				TargetPos:  1.0,
				Velocity:   0.0,
				DeltaTime:  1.0/60.0,
			},
			capture.SpringOutput{
				NewPos:      pos,
				NewVelocity: vel,
			},
		)
	}

	// Test 9: Multi-step convergence (10 steps)
	{
		spring := harmonica.NewSpring(harmonica.FPS(60), 6.0, 1.0)
		pos := 0.0
		vel := 0.0
		steps := make([]map[string]float64, 10)
		for i := 0; i < 10; i++ {
			pos, vel = spring.Update(pos, 1.0, 1.0/60.0)
			spring.SetVelocity(vel)
			steps[i] = map[string]float64{"pos": pos, "vel": vel}
		}
		fs.AddTestWithNotes("spring_convergence_10_steps",
			map[string]interface{}{
				"frequency":  6.0,
				"damping":    1.0,
				"start_pos":  0.0,
				"target_pos": 1.0,
				"steps":      10,
			},
			steps,
			"Tracks spring position over 10 simulation steps",
		)
	}

	// Test 10: Negative target (moving away from origin)
	{
		spring := harmonica.NewSpring(harmonica.FPS(60), 6.0, 1.0)
		pos, vel := spring.Update(0.0, -1.0, 1.0/60.0)
		fs.AddTestWithCategory("spring_negative_target", "unit",
			capture.SpringInput{
				Frequency: 6.0,
				Damping:   1.0,
				CurrentPos: 0.0,
				TargetPos:  -1.0,
				Velocity:   0.0,
				DeltaTime:  1.0/60.0,
			},
			capture.SpringOutput{
				NewPos:      pos,
				NewVelocity: vel,
			},
		)
	}
}

func captureProjectileTests(fs *capture.FixtureSet) {
	// Test 1: Projectile falling straight down
	{
		proj := harmonica.NewProjectile(harmonica.FPS(60), harmonica.Point{X: 0, Y: 10, Z: 0})
		proj.Update()
		pos := proj.Position()
		vel := proj.Velocity()
		fs.AddTestWithCategory("projectile_freefall", "unit",
			capture.ProjectileInput{
				X: 0, Y: 10, Z: 0,
				VelX: 0, VelY: 0, VelZ: 0,
				Gravity:   9.81,
				DeltaTime: 1.0/60.0,
			},
			capture.ProjectileOutput{
				X: pos.X, Y: pos.Y, Z: pos.Z,
				VelX: vel.X, VelY: vel.Y, VelZ: vel.Z,
			},
		)
	}

	// Test 2: Projectile with initial horizontal velocity
	{
		proj := harmonica.NewProjectile(harmonica.FPS(60), harmonica.Point{X: 0, Y: 10, Z: 0})
		proj.SetVelocity(harmonica.Vector{X: 5, Y: 0, Z: 0})
		proj.Update()
		pos := proj.Position()
		vel := proj.Velocity()
		fs.AddTestWithCategory("projectile_horizontal", "unit",
			capture.ProjectileInput{
				X: 0, Y: 10, Z: 0,
				VelX: 5, VelY: 0, VelZ: 0,
				Gravity:   9.81,
				DeltaTime: 1.0/60.0,
			},
			capture.ProjectileOutput{
				X: pos.X, Y: pos.Y, Z: pos.Z,
				VelX: vel.X, VelY: vel.Y, VelZ: vel.Z,
			},
		)
	}

	// Test 3: Projectile launched upward
	{
		proj := harmonica.NewProjectile(harmonica.FPS(60), harmonica.Point{X: 0, Y: 0, Z: 0})
		proj.SetVelocity(harmonica.Vector{X: 0, Y: 10, Z: 0})
		proj.Update()
		pos := proj.Position()
		vel := proj.Velocity()
		fs.AddTestWithCategory("projectile_upward", "unit",
			capture.ProjectileInput{
				X: 0, Y: 0, Z: 0,
				VelX: 0, VelY: 10, VelZ: 0,
				Gravity:   9.81,
				DeltaTime: 1.0/60.0,
			},
			capture.ProjectileOutput{
				X: pos.X, Y: pos.Y, Z: pos.Z,
				VelX: vel.X, VelY: vel.Y, VelZ: vel.Z,
			},
		)
	}

	// Test 4: 3D projectile motion
	{
		proj := harmonica.NewProjectile(harmonica.FPS(60), harmonica.Point{X: 1, Y: 2, Z: 3})
		proj.SetVelocity(harmonica.Vector{X: 1, Y: 2, Z: 3})
		proj.Update()
		pos := proj.Position()
		vel := proj.Velocity()
		fs.AddTestWithCategory("projectile_3d", "unit",
			capture.ProjectileInput{
				X: 1, Y: 2, Z: 3,
				VelX: 1, VelY: 2, VelZ: 3,
				Gravity:   9.81,
				DeltaTime: 1.0/60.0,
			},
			capture.ProjectileOutput{
				X: pos.X, Y: pos.Y, Z: pos.Z,
				VelX: vel.X, VelY: vel.Y, VelZ: vel.Z,
			},
		)
	}

	// Test 5: Multi-step trajectory
	{
		proj := harmonica.NewProjectile(harmonica.FPS(60), harmonica.Point{X: 0, Y: 0, Z: 0})
		proj.SetVelocity(harmonica.Vector{X: 10, Y: 15, Z: 0})
		steps := make([]map[string]float64, 10)
		for i := 0; i < 10; i++ {
			proj.Update()
			pos := proj.Position()
			vel := proj.Velocity()
			steps[i] = map[string]float64{
				"x": pos.X, "y": pos.Y, "z": pos.Z,
				"vx": vel.X, "vy": vel.Y, "vz": vel.Z,
			}
		}
		fs.AddTestWithNotes("projectile_trajectory_10_steps",
			map[string]interface{}{
				"start_pos":  []float64{0, 0, 0},
				"start_vel":  []float64{10, 15, 0},
				"gravity":    9.81,
				"steps":      10,
			},
			steps,
			"Tracks projectile position over 10 simulation steps",
		)
	}
}

func captureFPSTests(fs *capture.FixtureSet) {
	// Test FPS utility function
	fpsValues := []int{30, 60, 120, 144, 240}
	for _, fps := range fpsValues {
		delta := harmonica.FPS(fps)
		fs.AddTestWithCategory(fmt.Sprintf("fps_%d", fps), "unit",
			map[string]int{"fps": fps},
			map[string]float64{"delta": delta},
		)
	}
}
